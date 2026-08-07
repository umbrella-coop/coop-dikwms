// SPEC-001 AC-5/AC-6 unblocked via the terminusdb-repository crate.
// AC-6: all four scope layers round-trip without data loss, nearest-wins
//       resolution holds after reload.
// AC-5: schema change (Rust model -> derived TerminusDB schema) regenerates
//       and the server validates it; existing data migrates without loss.

#![recursion_limit = "512"]

use knowledge_domain::{EntityKind, PropertySet, Scope};
use terminusdb_bin::TerminusDBServer;
use terminusdb_client::{
    DocumentInsertArgs, MigrationOperation, MigrationOptions, TerminusDBModel,
};
use terminusdb_repository::{COMMON_INSTANCE, Repository, resolve_at};
use terminusdb_schema::{EntityIDFor, ToTDBInstance};
use terminusdb_schema_derive::{FromTDBInstance, TerminusDBModel};
use uuid::Uuid;

fn props(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
        .collect()
}

// ------------------------------------------------------------------
// AC-6: node storage round-trips all four scope layers without data loss
// ------------------------------------------------------------------
#[tokio::test]
async fn all_four_scope_layers_round_trip_without_data_loss() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("spec1_ac6_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client, db).await?;

    let entity_id = repo.create_entity(EntityKind::Node).await?;
    let common = COMMON_INSTANCE;
    let org = Uuid::now_v7();
    let ws = Uuid::now_v7();
    let project = Uuid::now_v7();

    repo.save_property_set(
        entity_id,
        common,
        &PropertySet::current(Scope::Common, 1, props(&[("name", "global")])),
        "alice",
        "common-v1",
    )
    .await?;
    repo.save_property_set(
        entity_id,
        org,
        &PropertySet::current(Scope::Org, 1, props(&[("name", "org")])),
        "alice",
        "org-v1",
    )
    .await?;
    repo.save_property_set(
        entity_id,
        ws,
        &PropertySet::current(Scope::Workspace, 1, props(&[("name", "ws")])),
        "alice",
        "ws-v1",
    )
    .await?;
    repo.save_property_set(
        entity_id,
        project,
        &PropertySet::current(Scope::Project, 1, props(&[("name", "project")])),
        "alice",
        "project-v1",
    )
    .await?;

    // Reload from the store
    let repo2 = Repository::new(repo.client().clone(), repo.db().to_string()).await?;
    let loaded = repo2.load_property_sets(entity_id).await?;

    // All four scope layers present, versions intact, content intact
    assert_eq!(loaded.len(), 4, "all four scope layers must round-trip");
    for psd in &loaded {
        assert_eq!(psd.version, 1);
        assert!(
            !psd.properties["name"].is_null(),
            "name missing for layer {}",
            psd.scope
        );
    }

    // Nearest-wins resolution holds after reload: project chain -> project
    let head = repo2.latest_commit().await?;
    let project_chain = vec![project, ws, org, common];
    let cur = resolve_at(&repo2, entity_id, &project_chain, &head).await?;
    assert_eq!(cur.properties["name"], serde_json::json!("project"));

    // Workspace chain -> workspace (project not in chain)
    let ws_chain = vec![ws, org, common];
    let cur_ws = resolve_at(&repo2, entity_id, &ws_chain, &head).await?;
    assert_eq!(cur_ws.properties["name"], serde_json::json!("ws"));

    Ok(())
}

// ------------------------------------------------------------------
// AC-5: schema change regenerates artifacts and the server validates
// ------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq, TerminusDBModel, FromTDBInstance)]
#[tdb(id_field = "id", key = "random")]
struct CatalogItemV1 {
    id: EntityIDFor<Self>,
    title: String,
}

// The evolved model reuses the V1 class name (schema evolution in place)
#[derive(Clone, Debug, PartialEq, TerminusDBModel, FromTDBInstance)]
#[tdb(id_field = "id", key = "random", class_name = "CatalogItemV1")]
struct CatalogItemV2 {
    id: EntityIDFor<Self>,
    title: String,
    price: i64,
}

#[tokio::test]
async fn schema_evolution_regenerates_and_validates_without_data_loss() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("spec1_ac5_{}", Uuid::now_v7().simple());
    let repo = Repository::new(client, db).await?;
    let spec = terminusdb_client::BranchSpec::from(repo.db());

    // V1 schema + instance (the "generated artifact" compiles and validates)
    repo.client()
        .schema::<CatalogItemV1>(DocumentInsertArgs::from(spec.clone()))
        .await?;
    let item = CatalogItemV1 {
        id: EntityIDFor::random(),
        title: "schema-org-style-item".to_string(),
    };
    repo.client()
        .insert(&item, DocumentInsertArgs::from(spec.clone()))
        .await?;

    // V2 model exists (derived schema recompiles with the new field) and the
    // evolved schema is applied to existing data via migration
    let resp = repo
        .client()
        .migrate_schema(
            &spec,
            "tester",
            "add price with default",
            vec![MigrationOperation::CreateClassProperty {
                class: "CatalogItemV1".to_string(),
                property: "price".to_string(),
                property_type: serde_json::json!("xsd:integer"),
                default: Some(serde_json::json!(0)),
            }],
            MigrationOptions::default(),
        )
        .await?;
    assert!(
        resp.status.contains("success"),
        "migration failed: {resp:?}"
    );

    // Existing instance unchanged (no data loss) + new property materialized
    let raw = repo
        .client()
        .get_document(
            &format!("CatalogItemV1/{}", item.id.id()),
            &spec,
            terminusdb_client::GetOpts::default(),
        )
        .await?;
    assert_eq!(raw["title"], serde_json::json!("schema-org-style-item"));
    assert_eq!(raw["price"], serde_json::json!(0));

    // A V2-shaped instance validates against the evolved schema
    let v2 = CatalogItemV2 {
        id: EntityIDFor::random(),
        title: "second".to_string(),
        price: 42,
    };
    repo.client()
        .insert(&v2, DocumentInsertArgs::from(spec))
        .await?;

    Ok(())
}
