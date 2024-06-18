use anyhow::{Context, Result};
use std::fs;

const FILE: &'static str = "models/SampleComponent.xsd";

fn main() -> Result<()> {
    let txt = fs::read_to_string(FILE).expect("Should have been able to read the file");
    let doc = roxmltree::Document::parse(&txt)?;
    let elem = doc
        .descendants()
        .find(|n| n.attribute("vendor") == Some("accellera.org"))
        .context("vendor")?;
    println!("elem {:?}", elem);
    Ok(())
}
