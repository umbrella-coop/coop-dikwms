//! Docgen: regenerate OpenAPI + mdBook chapters from code and specs.
//! Usage: `cargo run -p api --example docgen`

use std::path::Path;

fn main() -> anyhow::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs");
    api::docs::generate(
        &root.join("book/src"),
        &root.join("specs"),
        &root.join("openapi/openapi.json"),
    )?;
    println!("docs generated under {}", root.display());
    Ok(())
}
