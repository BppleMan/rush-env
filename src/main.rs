use color_eyre::Result;
use rush_env::core::rush::Rush;
use rush_env::visitor::{Visitor, VisitorContext};
use rush_env::{init_backtrace, init_base_dir, init_log};

const TEMPLATE: &str = include_str!("../assets/template/rush.xml");

fn main() -> Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();
    init_log(&base_dir);

    unsafe {
        std::env::set_var("RUSH_DIR", "/Users/bppleman/Library/Mobile Documents/com~apple~CloudDocs/dotfiles");
    }

    let rush: Rush = quick_xml::de::from_str(TEMPLATE)?;
    let mut context = VisitorContext::default();
    rush.visit(&mut context)?;
    println!("{}", context.script);

    Ok(())
}
