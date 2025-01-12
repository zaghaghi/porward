use color_eyre::{eyre::eyre, Result};

pub trait BuilderState {}

pub trait StringListSelector {
    fn select(&mut self, title: String, options: Vec<String>) -> Option<(usize, String)>;
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
        // TODO: check for aws cli and session manager plugin
        Ok(PortForwarderBuilder {
            port_forwarder: self.port_forwarder,
            selector: self.selector,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Profile> {
    pub fn profile(mut self) -> Result<PortForwarderBuilder<Instance>> {
        // TODO: select a profile
        let available_profiles = vec![
            "profile1".to_string(),
            "profile2".to_string(),
            "profile3".to_string(),
            "profile4".to_string(),
            "profile5".to_string(),
            "profile6".to_string(),
            "profile7".to_string(),
        ];

        if let Some((_, profile_name)) = self
            .selector
            .select("Select Profile".into(), available_profiles)
        {
            self.port_forwarder.profile_name = Some(profile_name);
            Ok(PortForwarderBuilder {
                port_forwarder: self.port_forwarder,
                selector: self.selector,
                marker: std::marker::PhantomData,
            })
        } else {
            Err(eyre!("canceled by user"))
        }
    }
}

impl PortForwarderBuilder<Instance> {
    pub fn instance(mut self) -> Result<PortForwarderBuilder<DestinationType>> {
        // TODO: select an ec2 instance
        let instances = vec!["i-123456".to_string(), "i-123457".to_string()];

        if let Some((_, instance)) = self
            .selector
            .select("Select EC2 Instance".into(), instances)
        {
            self.port_forwarder.instance_id = Some(instance);
            Ok(PortForwarderBuilder {
                port_forwarder: self.port_forwarder,
                selector: self.selector,
                marker: std::marker::PhantomData,
            })
        } else {
            Err(eyre!("canceled by user"))
        }
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

        if let Some((_, dtype)) = self
            .selector
            .select("Select Destination Type".into(), destination_types)
        {
            self.port_forwarder.destination_type = Some(dtype);
            Ok(PortForwarderBuilder {
                port_forwarder: self.port_forwarder,
                selector: self.selector,
                marker: std::marker::PhantomData,
            })
        } else {
            Err(eyre!("canceled by user"))
        }
    }
}

impl PortForwarderBuilder<Destination> {
    pub fn destination(mut self) -> Result<PortForwarderBuilder<Ready>> {
        let destinations = vec!["123".to_string(), "124".to_string(), "125".to_string()];

        if let Some((_, host_name)) = self.selector.select("Select Host".into(), destinations) {
            self.port_forwarder.host_name = Some(host_name);
            Ok(PortForwarderBuilder {
                port_forwarder: self.port_forwarder,
                selector: self.selector,
                marker: std::marker::PhantomData,
            })
        } else {
            Err(eyre!("canceled by user"))
        }
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
