use std::vec::Vec;

use crate::{ModelObjectId, ObjectKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImmutableNode {
    id: ModelObjectId,
    kind: ObjectKind,
    previous: Option<ModelObjectId>,
    references: Vec<ModelObjectId>,
}

impl ImmutableNode {
    pub fn new(
        id: ModelObjectId,
        kind: ObjectKind,
        previous: Option<ModelObjectId>,
        references: Vec<ModelObjectId>,
    ) -> Result<Self, NodeRefError> {
        if previous == Some(id) || references.contains(&id) {
            return Err(NodeRefError::SelfReference);
        }

        let mut sorted = references.clone();
        sorted.sort_unstable();
        if sorted.windows(2).any(|window| window[0] == window[1]) {
            return Err(NodeRefError::DuplicateReference);
        }

        Ok(Self {
            id,
            kind,
            previous,
            references,
        })
    }

    pub const fn id(&self) -> ModelObjectId {
        self.id
    }

    pub const fn kind(&self) -> ObjectKind {
        self.kind
    }

    pub const fn previous(&self) -> Option<ModelObjectId> {
        self.previous
    }

    pub fn references(&self) -> &[ModelObjectId] {
        &self.references
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeRefError {
    SelfReference,
    DuplicateReference,
}
