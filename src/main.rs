use porwarder::PortForwarder;
use ratatui::TerminalOptions;

pub mod porwarder;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _terminal = ratatui::init_with_options(TerminalOptions {
        viewport: ratatui::Viewport::Inline(10),
    });
    let result = PortForwarder::builder()
        .setup()?
        .profile()?
        .instance()?
        .destination_type()?
        .destination()?
        .run();
    ratatui::restore();
    result
}
