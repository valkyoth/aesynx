use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{ImmutableNode, ModelObjectId};

#[derive(Debug, Default)]
pub struct ObjectGraph {
    nodes: BTreeMap<ModelObjectId, ImmutableNode>,
}

impl ObjectGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, node: ImmutableNode) -> Result<(), ObjectGraphError> {
        if self.nodes.contains_key(&node.id()) {
            return Err(ObjectGraphError::DuplicateId);
        }
        if let Some(previous) = node.previous()
            && !self.nodes.contains_key(&previous)
        {
            return Err(ObjectGraphError::MissingReference);
        }
        if node
            .references()
            .iter()
            .any(|reference| !self.nodes.contains_key(reference))
        {
            return Err(ObjectGraphError::MissingReference);
        }

        self.nodes.insert(node.id(), node);
        Ok(())
    }

    pub fn get(&self, id: ModelObjectId) -> Option<&ImmutableNode> {
        self.nodes.get(&id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn reachable_from(&self, root: ModelObjectId) -> Result<Reachability, ObjectGraphError> {
        let mut reached = BTreeSet::new();
        let mut pending = VecDeque::new();
        pending.push_back(root);

        while let Some(id) = pending.pop_front() {
            if !reached.insert(id) {
                continue;
            }

            let Some(node) = self.nodes.get(&id) else {
                return Err(ObjectGraphError::MissingReference);
            };
            if let Some(previous) = node.previous() {
                pending.push_back(previous);
            }
            for reference in node.references() {
                pending.push_back(*reference);
            }
        }

        Ok(Reachability { reached })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reachability {
    reached: BTreeSet<ModelObjectId>,
}

impl Reachability {
    pub fn contains(&self, id: ModelObjectId) -> bool {
        self.reached.contains(&id)
    }

    pub fn len(&self) -> usize {
        self.reached.len()
    }

    pub fn is_empty(&self) -> bool {
        self.reached.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectGraphError {
    DuplicateId,
    MissingReference,
}
