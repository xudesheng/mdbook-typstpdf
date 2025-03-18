// construct a default config and export it as a toml file to the target directory

use std::fs;


use mdbook_typstpdf::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let toml_str = toml::to_string(&config)?;
    // get the current directory
    let current_dir = std::env::current_dir()?;
    // write the toml string to the target folder under the current directory
    fs::write(current_dir.join("target/config.toml"), toml_str)?;
    Ok(())
}