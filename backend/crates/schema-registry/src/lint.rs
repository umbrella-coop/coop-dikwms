//! Tag-immutability linting between namespace versions.
//!
//! Protobuf wire compatibility rule: field tags are never changed or reused.
//! `lint_tags(prev, next)` verifies, per message, that every field keeps its
//! tag and every tag keeps its field.

use prost_types::FileDescriptorSet;
use std::collections::HashMap;

use crate::descriptor;

#[derive(Debug, PartialEq, Eq)]
pub enum TagViolation {
    /// A tag now names a different field than in the previous version.
    TagReusedForDifferentField {
        tag: i32,
        prev_field: String,
        next_field: String,
    },
    /// A field changed its tag between versions.
    TagChanged {
        field: String,
        prev_tag: i32,
        next_tag: i32,
    },
}

pub fn lint_tags(prev: &FileDescriptorSet, next: &FileDescriptorSet) -> Result<(), TagViolation> {
    for type_name in descriptor::type_names(prev) {
        let Some(prev_msg) = descriptor::message(prev, &type_name) else {
            continue;
        };
        let Some(next_msg) = descriptor::message(next, &type_name) else {
            continue;
        };
        let prev_by_name: HashMap<String, i32> =
            descriptor::field_tags(prev_msg).into_iter().collect();
        let next_by_name: HashMap<String, i32> =
            descriptor::field_tags(next_msg).into_iter().collect();
        let next_by_tag: HashMap<i32, String> = descriptor::field_tags(next_msg)
            .into_iter()
            .map(|(n, t)| (t, n))
            .collect();

        // Fields must keep their tags.
        for (name, prev_tag) in &prev_by_name {
            if let Some(next_tag) = next_by_name.get(name) {
                if next_tag != prev_tag {
                    return Err(TagViolation::TagChanged {
                        field: name.clone(),
                        prev_tag: *prev_tag,
                        next_tag: *next_tag,
                    });
                }
            }
        }
        // Tags must keep their fields (no reuse for a different field).
        let prev_by_tag: HashMap<i32, String> = descriptor::field_tags(prev_msg)
            .into_iter()
            .map(|(n, t)| (t, n))
            .collect();
        for (tag, prev_field) in &prev_by_tag {
            if let Some(next_field) = next_by_tag.get(tag) {
                if next_field != prev_field {
                    return Err(TagViolation::TagReusedForDifferentField {
                        tag: *tag,
                        prev_field: prev_field.clone(),
                        next_field: next_field.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::field_descriptor_proto::Type;
    use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto};

    fn msg(name: &str, fields: Vec<(&str, i32)>) -> DescriptorProto {
        DescriptorProto {
            name: Some(name.to_string()),
            field: fields
                .into_iter()
                .map(|(n, num)| FieldDescriptorProto {
                    name: Some(n.to_string()),
                    number: Some(num),
                    r#type: Some(Type::String as i32),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn fds(messages: Vec<DescriptorProto>) -> FileDescriptorSet {
        FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("t.proto".into()),
                package: Some("demo.v1".into()),
                message_type: messages,
                ..Default::default()
            }],
        }
    }

    // AC-2: changed/reused tag rejected
    #[test]
    fn reused_tag_is_rejected() {
        let prev = fds(vec![msg("Doc", vec![("name", 1)])]);
        let next = fds(vec![msg("Doc", vec![("url", 1), ("name", 2)])]);
        let err = lint_tags(&prev, &next).unwrap_err();
        assert!(
            matches!(
                err,
                TagViolation::TagReusedForDifferentField { .. } | TagViolation::TagChanged { .. }
            ),
            "expected a tag violation, got {err:?}"
        );
    }

    // AC-3: additive fields pass
    #[test]
    fn additive_fields_pass() {
        let prev = fds(vec![msg("Doc", vec![("name", 1)])]);
        let next = fds(vec![msg("Doc", vec![("name", 1), ("url", 2)])]);
        assert_eq!(lint_tags(&prev, &next), Ok(()));
    }

    #[test]
    fn tag_change_is_rejected() {
        let prev = fds(vec![msg("Doc", vec![("name", 1)])]);
        let next = fds(vec![msg("Doc", vec![("name", 2)])]);
        assert_eq!(
            lint_tags(&prev, &next),
            Err(TagViolation::TagChanged {
                field: "name".into(),
                prev_tag: 1,
                next_tag: 2,
            })
        );
    }
}
