// Tests for SPEC-009: Versioned Schema Registry — Core (v1)
// Integration tests against a real per-process TerminusDB 12.1 server.
// AC Coverage: AC-1, AC-4, AC-5, AC-7 (AC-2/AC-3/AC-6 are pure unit tests
// in lint.rs / core.rs)

#![recursion_limit = "512"]

use prost::Message;
use prost_types::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    field_descriptor_proto::Type,
};
use schema_registry::{RegistryError, SchemaRegistry};
use terminusdb_bin::TerminusDBServer;
use uuid::Uuid;

fn message(name: &str, fields: Vec<FieldDescriptorProto>) -> DescriptorProto {
    DescriptorProto {
        name: Some(name.to_string()),
        field: fields,
        ..Default::default()
    }
}

fn field(name: &str, number: i32, ty: Type) -> FieldDescriptorProto {
    FieldDescriptorProto {
        name: Some(name.to_string()),
        number: Some(number),
        r#type: Some(ty as i32),
        ..Default::default()
    }
}

fn descriptor(package: &str, messages: Vec<DescriptorProto>) -> FileDescriptorSet {
    FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some(format!("{package}.proto")),
            package: Some(package.to_string()),
            message_type: messages,
            ..Default::default()
        }],
    }
}

fn encode(fds: &FileDescriptorSet) -> Vec<u8> {
    fds.encode_to_vec()
}

// ------------------------------------------------------------------
// AC-1: Register and retrieve a namespace with its type list
// ------------------------------------------------------------------
#[tokio::test]
async fn register_and_retrieve_namespace_with_types() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("reg_ac1_{}", Uuid::now_v7().simple());
    let reg = SchemaRegistry::init(client, db).await?;

    let fds = descriptor(
        "org.acme.v1",
        vec![message("Widget", vec![field("name", 1, Type::String)])],
    );
    reg.register(&fds, "org.acme.v1", "1").await?;

    let doc = reg.get("org.acme.v1", "1").await?;
    assert!(doc.is_some());
    let doc = doc.unwrap();
    assert_eq!(doc.types, vec!["Widget".to_string()]);
    use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
    assert_eq!(
        B64.decode(&doc.descriptor_b64).unwrap().len(),
        encode(&fds).len()
    );
    Ok(())
}

// ------------------------------------------------------------------
// AC-4/AC-5: additionalType validation
// ------------------------------------------------------------------
#[tokio::test]
async fn additional_type_validation_resolves_against_registry() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("reg_ac45_{}", Uuid::now_v7().simple());
    let reg = SchemaRegistry::init(client, db).await?;

    // Registered type (seeded: org.schema.v1 contains Person)
    reg.validate_additional_type("terminusdb://schema/org.schema.v1/Person")
        .await?;

    // Unregistered type
    let err = reg
        .validate_additional_type("terminusdb://schema/org.schema.v1/NotAType")
        .await
        .unwrap_err();
    assert!(matches!(err, RegistryError::TypeNotFound(_)));

    Ok(())
}

// ------------------------------------------------------------------
// AC-7: Core + schema.org namespaces seeded on init
// ------------------------------------------------------------------
#[tokio::test]
async fn core_and_schema_org_namespaces_seed_on_init() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("reg_ac7_{}", Uuid::now_v7().simple());
    let reg = SchemaRegistry::init(client, db).await?;

    let core = reg.get("core.v1", "1").await?;
    assert!(core.is_some(), "core.v1 must be seeded");
    let core = core.unwrap();
    for t in [
        "Thing",
        "Node",
        "Edge",
        "Combo",
        "CreativeWork",
        "MediaObject",
        "Action",
    ] {
        assert!(core.types.contains(&t.to_string()), "missing core type {t}");
    }

    let org = reg.get("org.schema.v1", "1").await?;
    assert!(org.is_some(), "org.schema.v1 must be seeded");
    let org = org.unwrap();
    assert!(org.types.contains(&"Person".to_string()));
    assert!(org.types.contains(&"ReviewAction".to_string()));

    Ok(())
}

// ------------------------------------------------------------------
// AC-2/AC-3 (integration side): version linting on registration
// ------------------------------------------------------------------
#[tokio::test]
async fn tag_reuse_fails_but_additive_fields_pass() -> anyhow::Result<()> {
    let server = TerminusDBServer::test_instance().await?;
    let client = server.client().await?;
    let db = format!("reg_ac23_{}", Uuid::now_v7().simple());
    let reg = SchemaRegistry::init(client, db).await?;

    // v1: name = 1
    let v1 = descriptor(
        "demo.v1",
        vec![message("Doc", vec![field("name", 1, Type::String)])],
    );
    reg.register(&v1, "demo.v1", "1").await?;

    // v2 additive: url = 2 -> ok
    let v2 = descriptor(
        "demo.v1",
        vec![message(
            "Doc",
            vec![
                field("name", 1, Type::String),
                field("url", 2, Type::String),
            ],
        )],
    );
    reg.register(&v2, "demo.v1", "2").await?;

    // v3 tag reuse: url steals tag 1 -> rejected
    let v3 = descriptor(
        "demo.v1",
        vec![message(
            "Doc",
            vec![
                field("url", 1, Type::String),
                field("name", 2, Type::String),
            ],
        )],
    );
    let err = reg.register(&v3, "demo.v1", "3").await.unwrap_err();
    assert!(matches!(err, RegistryError::TagViolation(_)), "got {err:?}");

    Ok(())
}
