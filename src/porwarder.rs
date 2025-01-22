use aws_config::BehaviorVersion;
use aws_runtime::env_config;
use color_eyre::{eyre::eyre, Result};
use std::{
    fmt::{Display, Formatter},
    process::{Command, Stdio},
};

#[derive(Clone)]
pub enum Service {
    ApplicationLoadBalancer,
    Postgresql,
    Redis,
    Valkey,
}

pub trait BuilderState {}

pub trait StringListSelector {
    fn select(&mut self, title: String, options: Vec<String>) -> Result<(usize, String)>;
}

#[allow(unused)]
pub struct PortForwarder {
    profile_name: Option<String>,
    instance_id: Option<String>,
    service: Option<Service>,
    host_name: Option<String>,
    host_port: Option<String>,
    local_port: Option<String>,
    read_only: bool,
}

pub struct PortForwarderBuilder<S: BuilderState = Start> {
    port_forwarder: Box<PortForwarder>,
    selector: Box<dyn StringListSelector>,
    marker: std::marker::PhantomData<S>,
}

pub struct Start;
pub struct Profile;
pub struct Instance;
pub struct DestinationType;
pub struct Destination;
pub struct Ready;

impl BuilderState for Start {}
impl BuilderState for Profile {}
impl BuilderState for Instance {}
impl BuilderState for DestinationType {}
impl BuilderState for Destination {}
impl BuilderState for Ready {}

impl Display for Service {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Service::ApplicationLoadBalancer => {
                write!(f, "ApplicationLoadBalancer")
            }
            Service::Postgresql => {
                write!(f, "Postgresql")
            }
            Service::Redis => {
                write!(f, "Redis")
            }
            Service::Valkey => {
                write!(f, "Valkey")
            }
        }
    }
}

impl Service {
    fn default_port(&self) -> u16 {
        match self {
            Service::ApplicationLoadBalancer => 443,
            Service::Postgresql => 5432,
            Service::Redis => 6379,
            Service::Valkey => 6379,
        }
    }
}

impl PortForwarderBuilder<Start> {
    pub fn setup(self) -> Result<PortForwarderBuilder<Profile>> {
        Command::new("aws").arg("--version").output().map_err(|_| {
            eyre!("aws cli is not installed. Please install it before running this program.")
        })?;
        Command::new("session-manager-plugin")
            .arg("--version")
            .output()
            .map_err(|_| {
                eyre!("session-manager-plugin is not installed. Please install it before running this program.")
            })?;
        Ok(PortForwarderBuilder {
            port_forwarder: self.port_forwarder,
            selector: self.selector,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Profile> {
    pub async fn profile(mut self) -> Result<PortForwarderBuilder<Instance>> {
        let fs = aws_types::os_shim_internal::Fs::real();
        let env = aws_types::os_shim_internal::Env::real();
        let profile_files = env_config::file::EnvConfigFiles::default();
        let profiles_set = aws_config::profile::load(&fs, &env, &profile_files, None).await?;

        let available_profiles = profiles_set
            .profiles()
            .map(|name| name.to_string())
            .collect();

        let (_, profile_name) = self
            .selector
            .select("Select Profile".into(), available_profiles)?;

        self.port_forwarder.profile_name = Some(profile_name);
        Ok(PortForwarderBuilder {
            port_forwarder: self.port_forwarder,
            selector: self.selector,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Instance> {
    pub async fn instance(mut self) -> Result<PortForwarderBuilder<DestinationType>> {
        let profile_name = self
            .port_forwarder
            .profile_name
            .as_ref()
            .ok_or(eyre!("profile name is not set"))?;
        let config = aws_config::defaults(BehaviorVersion::latest())
            .profile_name(profile_name)
            .load()
            .await;
        let client = aws_sdk_ec2::Client::new(&config);
        let instances = client
            .describe_instances()
            .filters(
                aws_sdk_ec2::types::Filter::builder()
                    .name("instance-state-name")
                    .values("running")
                    .build(),
            )
            .send()
            .await?
            .reservations
            .unwrap_or(vec![])
            .iter()
            .flat_map(|reservation| {
                reservation.instances().iter().filter_map(|instance| {
                    if let Some(id) = instance.instance_id() {
                        if let Some(name) = instance
                            .tags()
                            .iter()
                            .find(|tag| tag.key().unwrap_or_default() == "Name")
                        {
                            Some((id.to_string(), name.value().unwrap_or_default().to_string()))
                        } else {
                            Some((id.to_string(), "".to_string()))
                        }
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        let (idx, _) = self.selector.select(
            "Select EC2 Instance".into(),
            instances
                .iter()
                .map(|(id, name)| format!("{} ({})", name, id))
                .collect(),
        )?;
        self.port_forwarder.instance_id = instances.get(idx).map(|(id, _)| id.clone()).clone();
        Ok(PortForwarderBuilder {
            port_forwarder: self.port_forwarder,
            selector: self.selector,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<DestinationType> {
    pub fn destination_type(mut self) -> Result<PortForwarderBuilder<Destination>> {
        let services = [
            Service::ApplicationLoadBalancer,
            Service::Redis,
            Service::Valkey,
            Service::Postgresql,
        ];

        let (idx, _) = self.selector.select(
            "Select Destination Type".into(),
            services.iter().map(|service| service.to_string()).collect(),
        )?;

        self.port_forwarder.service = services.get(idx).cloned();
        let default_port = self
            .port_forwarder
            .service
            .as_ref()
            .map(|service| service.default_port());
        self.port_forwarder.host_port = default_port.map(|port| port.to_string());
        self.port_forwarder.local_port = default_port.map(|port| {
            if port < 1000 {
                (port + 1000).to_string()
            } else {
                port.to_string()
            }
        });
        Ok(PortForwarderBuilder {
            port_forwarder: self.port_forwarder,
            selector: self.selector,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Destination> {
    pub async fn destination(mut self) -> Result<PortForwarderBuilder<Ready>> {
        let destinations = match self
            .port_forwarder
            .service
            .clone()
            .ok_or(eyre!("destination type is empty"))?
        {
            Service::ApplicationLoadBalancer => self.application_load_balancers().await?,
            Service::Postgresql => self.postgresql_servers().await?,
            Service::Redis => self.redis_servers()?,
            Service::Valkey => self.valkey_servers()?,
        };

        let (idx, _) = self.selector.select(
            "Select Host".into(),
            destinations
                .iter()
                .map(|(_, title)| title.to_owned())
                .collect(),
        )?;
        self.port_forwarder.host_name = destinations
            .get(idx)
            .map(|(host_name, _)| host_name.to_owned());

        Ok(PortForwarderBuilder {
            port_forwarder: self.port_forwarder,
            selector: self.selector,
            marker: std::marker::PhantomData,
        })
    }

    async fn application_load_balancers(&self) -> Result<Vec<(String, String)>> {
        let profile_name = self
            .port_forwarder
            .profile_name
            .as_ref()
            .ok_or(eyre!("profile name is not set"))?;
        let config = aws_config::defaults(BehaviorVersion::latest())
            .profile_name(profile_name)
            .load()
            .await;
        let client = aws_sdk_elasticloadbalancingv2::Client::new(&config);
        let response = client.describe_load_balancers().send().await?;
        Ok(response
            .load_balancers
            .unwrap_or(vec![])
            .iter()
            .filter_map(|lb| {
                lb.dns_name
                    .as_ref()
                    .map(|dns_name| {
                        (
                            dns_name.to_owned(),
                            lb.load_balancer_name.to_owned().unwrap_or(dns_name.clone()),
                        )
                    })
                    .clone()
            })
            .collect())
    }

    async fn postgresql_servers(&self) -> Result<Vec<(String, String)>> {
        let profile_name = self
            .port_forwarder
            .profile_name
            .as_ref()
            .ok_or(eyre!("profile name is not set"))?;
        let config = aws_config::defaults(BehaviorVersion::latest())
            .profile_name(profile_name)
            .load()
            .await;
        let client = aws_sdk_rds::Client::new(&config);
        let response = client.describe_db_cluster_endpoints().send().await?;
        Ok(response
            .db_cluster_endpoints
            .unwrap_or(vec![])
            .iter()
            .filter_map(|db_cluster_endpoint| {
                db_cluster_endpoint
                    .endpoint
                    .as_ref()
                    .map(|dns_name| {
                        (
                            dns_name.to_owned(),
                            db_cluster_endpoint
                                .endpoint
                                .to_owned()
                                .unwrap_or(dns_name.clone()),
                        )
                    })
                    .clone()
            })
            .collect())
    }

    fn redis_servers(&self) -> Result<Vec<(String, String)>> {
        Ok(vec![])
    }

    fn valkey_servers(&self) -> Result<Vec<(String, String)>> {
        Ok(vec![])
    }
}

impl PortForwarderBuilder<Ready> {
    pub fn build(self) -> Result<Box<PortForwarder>> {
        Ok(self.port_forwarder)
    }
}

impl PortForwarder {
    pub fn builder(selector: Box<dyn StringListSelector>) -> PortForwarderBuilder {
        PortForwarderBuilder {
            port_forwarder: Box::new(PortForwarder {
                profile_name: None,
                instance_id: None,
                service: None,
                host_name: None,
                host_port: None,
                local_port: None,
                read_only: true,
            }),
            selector,
            marker: std::marker::PhantomData,
        }
    }

    pub fn run(self) -> Result<()> {
        let profile_name = self
            .profile_name
            .as_ref()
            .ok_or(eyre!("profile name is not set"))?;
        let instance_id = self
            .instance_id
            .as_ref()
            .ok_or(eyre!("instance id is not set"))?;
        let host_name = self
            .host_name
            .as_ref()
            .ok_or(eyre!("host name is not set"))?;
        let host_port = self
            .host_port
            .as_ref()
            .ok_or(eyre!("host port is not set"))?;
        let local_port = self
            .local_port
            .as_ref()
            .ok_or(eyre!("local port is not set"))?;
        let command = format!(
            r#"aws --profile {} ssm start-session --target {} --document-name AWS-StartPortForwardingSessionToRemoteHost --parameters '{{"host":["{}"],"portNumber":["{}"], "localPortNumber":["{}"]}}'"#,
            profile_name, instance_id, host_name, host_port, local_port
        );
        ratatui::restore();
        println!("Running:\r\n{}", command);
        let mut child = Command::new("aws")
            .arg("--profile")
            .arg(profile_name)
            .arg("ssm")
            .arg("start-session")
            .arg("--target")
            .arg(instance_id)
            .arg("--document-name")
            .arg("AWS-StartPortForwardingSessionToRemoteHost")
            .arg("--parameters")
            .arg(format!(
                r#"{{"host":["{}"],"portNumber":["{}"], "localPortNumber":["{}"]}}"#,
                host_name, host_port, local_port
            ))
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        child.wait().map_err(|_| {
            eyre!(
                    r#"aws --profile {} ssm start-session --target {} --document-name AWS-StartPortForwardingSessionToRemoteHost --parameters '{{"host":["{}"],"portNumber":["{}"], "localPortNumber":["{}"]}}'"#,
                    profile_name,
                    instance_id,
                    host_name,
                    host_port,
                    local_port
                )
        })?;
        Ok(())
    }
}
