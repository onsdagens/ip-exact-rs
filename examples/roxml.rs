use anyhow::{Context, Result};

fn main() -> Result<()> {
    let doc = roxmltree::Document::parse("<rect id='rect1'/>")?;
    let elem = doc
        .descendants()
        .find(|n| n.attribute("id") == Some("rect1"))
        .context("no rect1")?;
    assert!(elem.has_tag_name("rect"));
    Ok(())
}
