//! FileDescriptorSet helpers: package/type/field extraction.

use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet};

pub fn package(fds: &FileDescriptorSet) -> Option<String> {
    fds.file
        .first()
        .and_then(|f: &FileDescriptorProto| f.package.clone())
}

/// All message type names defined by the descriptor set.
pub fn type_names(fds: &FileDescriptorSet) -> Vec<String> {
    fds.file
        .iter()
        .flat_map(|f: &FileDescriptorProto| f.message_type.iter())
        .filter_map(|m: &DescriptorProto| m.name.clone())
        .collect()
}

pub fn message<'a>(fds: &'a FileDescriptorSet, name: &str) -> Option<&'a DescriptorProto> {
    fds.file
        .iter()
        .flat_map(|f: &FileDescriptorProto| f.message_type.iter())
        .find(|m| m.name.as_deref() == Some(name))
}

/// (field name -> tag) map for a message.
pub fn field_tags(msg: &DescriptorProto) -> Vec<(String, i32)> {
    msg.field
        .iter()
        .filter_map(|f: &FieldDescriptorProto| Some((f.name.clone()?, f.number?)))
        .collect()
}

/// Core definition source of truth: the canonical `core.proto` artifact lives
/// at schemas/proto/core/v1/core.proto and mirrors `core::core_v1()`.
#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn type_names_extracts_all_messages() {
        let fds = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("t.proto".into()),
                package: Some("demo.v1".into()),
                message_type: vec![
                    DescriptorProto {
                        name: Some("A".into()),
                        ..Default::default()
                    },
                    DescriptorProto {
                        name: Some("B".into()),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
        };
        assert_eq!(type_names(&fds), vec!["A".to_string(), "B".to_string()]);
        assert_eq!(package(&fds).as_deref(), Some("demo.v1"));
    }

    #[test]
    fn descriptor_set_round_trips_through_bytes() {
        let fds = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("t.proto".into()),
                package: Some("demo.v1".into()),
                message_type: vec![DescriptorProto {
                    name: Some("A".into()),
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let bytes = fds.encode_to_vec();
        let decoded = FileDescriptorSet::decode(bytes.as_slice()).unwrap();
        assert_eq!(type_names(&decoded), vec!["A".to_string()]);
    }
}
