use porwarder::PortForwarder;
use selector::TUIStringListSelector;

pub mod porwarder;
pub mod selector;

async fn run() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let selector = Box::new(TUIStringListSelector::inline_view(6));
    let result = PortForwarder::builder(selector)
        .setup()?
        .profile()
        .await?
        .instance()
        .await?
        .destination_type()?
        .destination()
        .await?
        .build()?
        .run();
    ratatui::restore();
    result
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    if let Err(e) = run().await {
        ratatui::restore();
        println!("{}{}", e, " ".repeat(80));
    }
    Ok(())
}
