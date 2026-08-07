//! OpenAPI + mdBook documentation generation (SPEC-016).

use std::fs;
use std::path::Path;

use utoipa::OpenApi;

/// Regenerate documentation artifacts from code + specs.
/// - `openapi.json` from the ApiDoc
/// - `docs/book/src/api-reference.md`, `specs.md`, `registry.md`
pub fn generate(book_src: &Path, spec_dir: &Path, openapi_out: &Path) -> anyhow::Result<()> {
    let doc = crate::ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&doc)?;
    fs::create_dir_all(openapi_out.parent().unwrap_or(Path::new(".")))?;
    fs::write(openapi_out, json.clone())?;

    // API reference chapter
    let api_ref = format!(
        "# API Reference\n\nGenerated from the axum handlers (utoipa).\n\n- Interactive: `GET /openapi.json`\n- Swagger UI: `/swagger`\n\n```json\n{json}\n```\n"
    );
    fs::write(book_src.join("api-reference.md"), api_ref)?;

    // Specs index chapter (AC traceability)
    let mut specs = String::from("# Specs\n\nIndex generated from `docs/specs/`.\n\n");
    let mut entries: Vec<String> = Vec::new();
    for entry in fs::read_dir(spec_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("SPEC-") || !name.ends_with(".md") {
            continue;
        }
        let content = fs::read_to_string(entry.path())?;
        let title = content
            .lines()
            .find(|l| l.starts_with("# SPEC-"))
            .unwrap_or(&name)
            .trim_start_matches("# ")
            .to_string();
        let status = content
            .lines()
            .find(|l| l.contains("status:"))
            .map(|l| {
                l.trim_start_matches("<!--")
                    .trim_end_matches("-->")
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();
        let acs = content.matches("AC-").count();
        entries.push(format!(
            "- **{title}** ({status}) — {acs} AC references — [source](../../specs/{name})"
        ));
    }
    entries.sort();
    specs.push_str(&entries.join("\n"));
    specs.push('\n');
    fs::write(book_src.join("specs.md"), specs)?;

    // Registry chapter (snapshot fallback — needs a live DB otherwise)
    let registry_snapshot = openapi_out
        .parent()
        .unwrap_or(Path::new("."))
        .join("registry.json");
    let registry = if registry_snapshot.exists() {
        let data = fs::read_to_string(&registry_snapshot)?;
        format!("# Schema Registry\n\nSnapshot of registered namespaces:\n\n```json\n{data}\n```\n")
    } else {
        String::from(
            "# Schema Registry\n\nNamespace listing requires a running TerminusDB fixture.\n\nRun: `cargo run -p api --example docgen -- --registry` to write the snapshot.\n",
        )
    };
    fs::write(book_src.join("registry.md"), registry)?;

    Ok(())
}
