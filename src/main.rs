use porwarder::PortForwarder;
use selector::TUIStringListSelector;

pub mod porwarder;
pub mod selector;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let selector = Box::new(TUIStringListSelector::inline_view(6));
    let result = PortForwarder::builder(selector)
        .setup()?
        .profile()?
        .instance()?
        .destination_type()?
        .destination()?
        .build()?
        .run();
    ratatui::restore();
    result
}
