use aws_config::BehaviorVersion;
use aws_runtime::env_config;
use color_eyre::{eyre::eyre, Result};
use std::process::Command;

pub trait BuilderState {}

pub trait StringListSelector {
    fn select(&mut self, title: String, options: Vec<String>) -> Result<(usize, String)>;
}

#[allow(unused)]
pub struct PortForwarder {
    profile_name: Option<String>,
    instance_id: Option<String>,
    destination_type: Option<String>,
    host_name: Option<String>,
    host_port: Option<String>,
    local_port: Option<String>,
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
            .clone()
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
        let destination_types = vec![
            "Application Load Balancer".to_string(),
            "Redis".to_string(),
            "Valkey".to_string(),
            "Postgresql".to_string(),
        ];

        let (_, dtype) = self
            .selector
            .select("Select Destination Type".into(), destination_types)?;

        self.port_forwarder.destination_type = Some(dtype);
        Ok(PortForwarderBuilder {
            port_forwarder: self.port_forwarder,
            selector: self.selector,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Destination> {
    pub fn destination(mut self) -> Result<PortForwarderBuilder<Ready>> {
        let destinations = vec!["123".to_string(), "124".to_string(), "125".to_string()];

        let (_, host_name) = self.selector.select("Select Host".into(), destinations)?;
        self.port_forwarder.host_name = Some(host_name);
        Ok(PortForwarderBuilder {
            port_forwarder: self.port_forwarder,
            selector: self.selector,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Ready> {
    pub fn build(self) -> Result<Box<PortForwarder>> {
        // TODO: check all options are some
        Ok(self.port_forwarder)
    }
}

impl PortForwarder {
    pub fn builder(selector: Box<dyn StringListSelector>) -> PortForwarderBuilder {
        PortForwarderBuilder {
            port_forwarder: Box::new(PortForwarder {
                profile_name: None,
                instance_id: None,
                destination_type: None,
                host_name: None,
                host_port: None,
                local_port: None,
            }),
            selector,
            marker: std::marker::PhantomData,
        }
    }

    pub fn run(self) -> Result<()> {
        Ok(())
    }
}
