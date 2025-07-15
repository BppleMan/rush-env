use color_eyre::Result;
use rush_env::core::rush::Rush;

const TEMPLATE: &str = include_str!("../assets/template/rush.xml");

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let rush: Rush = quick_xml::de::from_str(TEMPLATE)?;
    println!("{:#?}", rush);

    Ok(())
}
