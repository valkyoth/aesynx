use crate::{
    ImmutableNode, ModelObjectId, NodeRefError, ObjectGraph, ObjectGraphError, ObjectKind,
};

fn id(value: u128) -> Result<ModelObjectId, String> {
    ModelObjectId::new(value).map_err(|error| format!("invalid test id: {error:?}"))
}

fn node(
    value: u128,
    kind: ObjectKind,
    previous: Option<ModelObjectId>,
    references: Vec<ModelObjectId>,
) -> Result<ImmutableNode, String> {
    ImmutableNode::new(id(value)?, kind, previous, references)
        .map_err(|error| format!("invalid test node: {error:?}"))
}

#[test]
fn object_id_rejects_zero_and_redacts_debug() -> Result<(), String> {
    assert!(ModelObjectId::new(0).is_err());

    let rendered = format!("{:?}", id(0xfeed_cafe)?);
    assert!(rendered.contains("<redacted>"));
    assert!(!rendered.contains("feed"));
    Ok(())
}

#[test]
fn immutable_node_rejects_self_and_duplicate_references() -> Result<(), String> {
    let root = id(1)?;
    assert_eq!(
        ImmutableNode::new(root, ObjectKind::SnapshotRoot, Some(root), Vec::new()),
        Err(NodeRefError::SelfReference)
    );
    assert_eq!(
        ImmutableNode::new(root, ObjectKind::SnapshotRoot, None, vec![root]),
        Err(NodeRefError::SelfReference)
    );

    let child = id(2)?;
    assert_eq!(
        ImmutableNode::new(root, ObjectKind::SnapshotRoot, None, vec![child, child]),
        Err(NodeRefError::DuplicateReference)
    );
    Ok(())
}

#[test]
fn graph_insert_rejects_duplicates_and_missing_references() -> Result<(), String> {
    let mut graph = ObjectGraph::new();
    graph
        .insert(node(1, ObjectKind::Memory, None, Vec::new())?)
        .map_err(|error| format!("insert failed: {error:?}"))?;

    assert_eq!(
        graph.insert(node(1, ObjectKind::Endpoint, None, Vec::new())?),
        Err(ObjectGraphError::DuplicateId)
    );
    assert_eq!(
        graph.insert(node(2, ObjectKind::Queue, Some(id(9)?), Vec::new())?),
        Err(ObjectGraphError::MissingReference)
    );
    assert_eq!(
        graph.insert(node(3, ObjectKind::Queue, None, vec![id(9)?])?),
        Err(ObjectGraphError::MissingReference)
    );
    Ok(())
}

#[test]
fn reachability_walks_references_and_previous_nodes() -> Result<(), String> {
    let mut graph = ObjectGraph::new();
    graph
        .insert(node(1, ObjectKind::Memory, None, Vec::new())?)
        .map_err(|error| format!("insert root failed: {error:?}"))?;
    graph
        .insert(node(2, ObjectKind::Queue, None, vec![id(1)?])?)
        .map_err(|error| format!("insert queue failed: {error:?}"))?;
    graph
        .insert(node(3, ObjectKind::SnapshotRoot, Some(id(2)?), Vec::new())?)
        .map_err(|error| format!("insert snapshot failed: {error:?}"))?;

    let reached = graph
        .reachable_from(id(3)?)
        .map_err(|error| format!("reachability failed: {error:?}"))?;

    assert_eq!(reached.len(), 3);
    assert!(reached.contains(id(1)?));
    assert!(reached.contains(id(2)?));
    assert!(reached.contains(id(3)?));
    Ok(())
}

#[test]
fn graph_lookup_exposes_immutable_node_metadata() -> Result<(), String> {
    let mut graph = ObjectGraph::new();
    let root = node(1, ObjectKind::WorldFact, None, Vec::new())?;
    graph
        .insert(root)
        .map_err(|error| format!("insert failed: {error:?}"))?;

    let Some(loaded) = graph.get(id(1)?) else {
        return Err(String::from("inserted object not found"));
    };

    assert_eq!(loaded.kind(), ObjectKind::WorldFact);
    assert!(loaded.previous().is_none());
    assert!(loaded.references().is_empty());
    assert!(!ObjectKind::WorldFact.is_service_backed());
    assert!(ObjectKind::Queue.is_service_backed());
    Ok(())
}
