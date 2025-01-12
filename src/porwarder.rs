use color_eyre::Result;

pub trait BuilderState {}

#[derive(Default)]
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
    parameters: Box<PortForwarder>,
    marker: std::marker::PhantomData<S>,
}

pub struct Start;
pub struct Profile;
pub struct Instance;
pub struct DestinationType;
pub struct Destination;

impl BuilderState for Start {}
impl BuilderState for Profile {}
impl BuilderState for Instance {}
impl BuilderState for DestinationType {}
impl BuilderState for Destination {}

impl PortForwarderBuilder<Start> {
    pub fn setup(self) -> Result<PortForwarderBuilder<Profile>> {
        // TODO: check for aws cli and session manager plugin
        Ok(PortForwarderBuilder {
            parameters: self.parameters,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Profile> {
    pub fn profile(self) -> Result<PortForwarderBuilder<Instance>> {
        // TODO: select a profile
        Ok(PortForwarderBuilder {
            parameters: self.parameters,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Instance> {
    pub fn instance(self) -> Result<PortForwarderBuilder<DestinationType>> {
        // TODO: select an ec2 instance
        Ok(PortForwarderBuilder {
            parameters: self.parameters,
            marker: std::marker::PhantomData,
        })
    }
}
impl PortForwarderBuilder<DestinationType> {
    pub fn destination_type(self) -> Result<PortForwarderBuilder<Destination>> {
        // TODO: select a destination type
        Ok(PortForwarderBuilder {
            parameters: self.parameters,
            marker: std::marker::PhantomData,
        })
    }
}

impl PortForwarderBuilder<Destination> {
    pub fn destination(self) -> Result<Box<PortForwarder>> {
        // TODO: select a destination
        Ok(self.parameters)
    }
}

impl PortForwarder {
    pub fn builder() -> PortForwarderBuilder {
        PortForwarderBuilder {
            parameters: Box::new(PortForwarder::default()),
            marker: std::marker::PhantomData,
        }
    }
    pub fn run(self) -> Result<()> {
        Ok(())
    }
}
