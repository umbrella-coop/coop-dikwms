//! Seeded namespace descriptors: `core.v1` and `org.schema.v1`.
//!
//! These are constructed via `prost_types`, mirroring the canonical
//! `schemas/proto/core/v1/core.proto` artifact. Single construction site —
//! the lint test keeps them honest.

use prost_types::field_descriptor_proto::Type;
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet};

/// Core fields must stay in tags 1..=CORE_TAG_MAX (varint-friendly).
pub const CORE_TAG_MAX: i32 = 15;

fn f(name: &str, number: i32, ty: Type) -> FieldDescriptorProto {
    FieldDescriptorProto {
        name: Some(name.to_string()),
        number: Some(number),
        r#type: Some(ty as i32),
        ..Default::default()
    }
}

fn msg(name: &str, fields: Vec<FieldDescriptorProto>) -> DescriptorProto {
    DescriptorProto {
        name: Some(name.to_string()),
        field: fields,
        ..Default::default()
    }
}

/// Thing base: tags 1-4 (schema.org minimal property set + the extension seam).
fn thing_fields() -> Vec<FieldDescriptorProto> {
    vec![
        f("name", 1, Type::String),
        f("identifier", 2, Type::String),
        f("url", 3, Type::String),
        f("additionalType", 4, Type::String).label_repeated(),
    ]
}

trait Repeated {
    fn label_repeated(self) -> Self;
}
impl Repeated for FieldDescriptorProto {
    fn label_repeated(mut self) -> Self {
        self.label = Some(prost_types::field_descriptor_proto::Label::Repeated as i32);
        self
    }
}

/// `core.v1`: Thing-based kinds. Every kind embeds the Thing fields (1-4).
pub fn core_v1() -> FileDescriptorSet {
    let thing = msg("Thing", thing_fields());
    let node = msg("Node", thing_fields());
    let edge = msg(
        "Edge",
        thing_fields()
            .into_iter()
            .chain([
                f("subject", 5, Type::String),
                f("object", 6, Type::String),
                f("relationship", 7, Type::String),
            ])
            .collect(),
    );
    let combo = msg(
        "Combo",
        thing_fields()
            .into_iter()
            .chain([f("members", 5, Type::String).label_repeated()])
            .collect(),
    );
    let creative_work = msg("CreativeWork", thing_fields());
    let media_object = msg("MediaObject", thing_fields());
    let action = msg(
        "Action",
        thing_fields()
            .into_iter()
            .chain([f("agent", 5, Type::String), f("object", 6, Type::String)])
            .collect(),
    );
    FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("core/v1/core.proto".into()),
            package: Some("core.v1".into()),
            message_type: vec![
                thing,
                node,
                edge,
                combo,
                creative_work,
                media_object,
                action,
            ],
            ..Default::default()
        }],
    }
}

/// `org.schema.v1`: curated minimal schema.org subset (name-only in v1).
pub fn org_schema_v1() -> FileDescriptorSet {
    let types = [
        "Thing",
        "Person",
        "Organization",
        "CreativeWork",
        "Report",
        "MediaObject",
        "Action",
        "AssessAction",
        "ChooseAction",
        "VoteAction",
        "IgnoreAction",
        "ReactAction",
        "AgreeAction",
        "DisagreeAction",
        "DislikeAction",
        "EndorseAction",
        "LikeAction",
        "WantAction",
        "ReviewAction",
    ];
    FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("org/schema/v1/schema.org.proto".into()),
            package: Some("org.schema.v1".into()),
            message_type: types
                .iter()
                .map(|t| msg(t, vec![f("name", 1, Type::String)]))
                .collect(),
            ..Default::default()
        }],
    }
}

/// `git.v1`: git-domain kinds (SPEC-027 REQ-007). Field names mirror the
/// importer's property keys exactly — the bulk-endpoint lint compares
/// entity properties against these message fields (warn-not-fail).
pub fn git_v1() -> FileDescriptorSet {
    let author = msg(
        "Author",
        vec![f("name", 1, Type::String), f("email", 2, Type::String)],
    );
    let commit = msg(
        "Commit",
        vec![
            f("hash", 1, Type::String),
            f("message", 2, Type::String),
            f("authored_at", 3, Type::String),
            f("author", 4, Type::String),
            f("author_name", 5, Type::String),
            f("author_email", 6, Type::String),
            f("parent_hexshas", 7, Type::String).label_repeated(),
            f("parent_uuids", 8, Type::String).label_repeated(),
            f("paths", 9, Type::String).label_repeated(),
        ],
    );
    FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("git/v1/git.proto".into()),
            package: Some("git.v1".into()),
            message_type: vec![author, commit],
            ..Default::default()
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor;
    use std::collections::HashSet;

    // AC-6: core tags are 1..=15, no reuse, kinds embed the Thing base
    #[test]
    fn core_definitions_lint_clean() {
        let core = core_v1();
        let thing = descriptor::message(&core, "Thing").expect("Thing");
        let thing_fields: Vec<(String, i32)> = descriptor::field_tags(thing);

        for (_, tag) in &thing_fields {
            assert!(
                (1..=CORE_TAG_MAX).contains(tag),
                "Thing tag {tag} outside 1..={CORE_TAG_MAX}"
            );
        }
        let mut seen = HashSet::new();
        for (name, tag) in &thing_fields {
            assert!(seen.insert(tag), "tag {tag} reused in Thing");
            assert!(!name.is_empty());
        }

        // Every kind embeds the Thing fields with identical tags
        for kind in [
            "Node",
            "Edge",
            "Combo",
            "CreativeWork",
            "MediaObject",
            "Action",
        ] {
            let m = descriptor::message(&core, kind).expect(kind);
            let tags: HashSet<i32> = descriptor::field_tags(m)
                .into_iter()
                .map(|(_, t)| t)
                .collect();
            for (_, tag) in &thing_fields {
                assert!(tags.contains(tag), "{kind} missing Thing tag {tag}");
            }
        }
    }

    #[test]
    fn org_schema_seed_covers_assess_action_tree() {
        let names: HashSet<String> = descriptor::type_names(&org_schema_v1())
            .into_iter()
            .collect();
        for t in [
            "Person",
            "Action",
            "AssessAction",
            "VoteAction",
            "ReviewAction",
            "EndorseAction",
        ] {
            assert!(names.contains(t), "missing {t}");
        }
    }

    #[test]
    fn git_v1_defines_author_and_commit() {
        let git = git_v1();
        let author = descriptor::message(&git, "Author").expect("Author");
        assert_eq!(
            descriptor::field_tags(author),
            vec![("name".to_string(), 1), ("email".to_string(), 2)]
        );
        let commit = descriptor::message(&git, "Commit").expect("Commit");
        let tags: HashSet<String> = descriptor::field_tags(commit)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        for field in [
            "hash",
            "message",
            "authored_at",
            "author",
            "author_name",
            "author_email",
            "parent_hexshas",
            "parent_uuids",
            "paths",
        ] {
            assert!(tags.contains(field), "Commit missing {field}");
        }
        let mut seen = HashSet::new();
        for (_, tag) in descriptor::field_tags(commit) {
            assert!(seen.insert(tag), "tag {tag} reused in Commit");
        }
    }
}
