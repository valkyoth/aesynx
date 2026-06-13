use alloc::format;

use aesynx_abi::{CoreId, ObjectId, PrincipalId};
use aesynx_cap::{CapKind, CapPerms, CapabilityTable};

use crate::{KernelObject, ObjectCreate, ObjectRegistry, ObjectRegistryError};

fn object_id(value: u128) -> ObjectId {
    ObjectId::new(value)
}

fn owner(value: u32) -> CoreId {
    CoreId::new(value)
}

#[test]
fn registry_creates_lists_and_deletes_local_objects() -> Result<(), ObjectRegistryError> {
    let mut registry = ObjectRegistry::<4>::new();
    let memory = registry.create(ObjectCreate::memory(object_id(1), owner(0)))?;
    let endpoint = registry.create(ObjectCreate::endpoint(object_id(2), owner(0)))?;
    let queue = registry.create(ObjectCreate::queue(object_id(3), owner(1)))?;
    let task = registry.create(ObjectCreate::task_placeholder(object_id(4), owner(1)))?;

    assert_eq!(registry.len(), 4);
    assert_eq!(memory.object_id(), object_id(1));
    assert_eq!(endpoint.owner_core(), owner(0));
    assert_eq!(queue.owner_core(), owner(1));
    assert_eq!(task.generation(), 1);

    let mut listed = [memory; 4];
    let listed_count = registry.list(&mut listed)?;
    assert_eq!(listed_count, 4);
    assert!(
        listed
            .iter()
            .any(|record| record.object_id() == object_id(4))
    );

    let deleted = registry.delete(object_id(3))?;
    assert_eq!(deleted.object_id(), object_id(3));
    assert_eq!(registry.len(), 3);
    assert_eq!(
        registry.get(object_id(3)),
        Err(ObjectRegistryError::ObjectNotFound)
    );
    let recreated = registry.create(ObjectCreate::queue(object_id(3), owner(1)))?;
    assert_eq!(recreated.object_id(), object_id(3));
    assert_eq!(recreated.generation(), deleted.generation() + 1);
    assert_eq!(registry.len(), 4);

    Ok(())
}

#[test]
fn registry_recycles_deleted_slots_with_new_generations() -> Result<(), ObjectRegistryError> {
    let mut registry = ObjectRegistry::<1>::new();
    let first = registry.create(ObjectCreate::endpoint(object_id(9), owner(0)))?;
    let deleted = registry.delete(first.object_id())?;

    assert_eq!(deleted, first);
    assert!(registry.is_empty());

    let second = registry.create(ObjectCreate::endpoint(object_id(9), owner(0)))?;

    assert_eq!(second.object_id(), first.object_id());
    assert_eq!(second.generation(), first.generation() + 1);
    Ok(())
}

#[test]
fn registry_rejects_invalid_duplicate_and_over_capacity_objects() -> Result<(), ObjectRegistryError>
{
    let mut registry = ObjectRegistry::<1>::new();

    assert_eq!(
        registry.create(ObjectCreate::memory(ObjectId::new(0), owner(0))),
        Err(ObjectRegistryError::InvalidObjectId)
    );

    registry.create(ObjectCreate::memory(object_id(1), owner(0)))?;
    assert_eq!(
        registry.create(ObjectCreate::endpoint(object_id(1), owner(0))),
        Err(ObjectRegistryError::DuplicateObject)
    );
    assert_eq!(
        registry.create(ObjectCreate::endpoint(object_id(2), owner(0))),
        Err(ObjectRegistryError::RegistryFull)
    );

    Ok(())
}

#[test]
fn registry_list_is_validate_then_commit() -> Result<(), ObjectRegistryError> {
    let mut registry = ObjectRegistry::<2>::new();
    let memory = registry.create(ObjectCreate::memory(object_id(1), owner(0)))?;
    registry.create(ObjectCreate::endpoint(object_id(2), owner(0)))?;
    let mut too_small = [memory; 1];

    assert_eq!(
        registry.list(&mut too_small),
        Err(ObjectRegistryError::OutputTooSmall)
    );
    assert_eq!(too_small[0], memory);

    Ok(())
}

#[test]
fn object_capabilities_resolve_to_live_registry_objects() -> Result<(), ObjectRegistryError> {
    let mut registry = ObjectRegistry::<2>::new();
    let memory = registry.create(ObjectCreate::memory(object_id(1), owner(0)))?;
    let mut table = CapabilityTable::<2>::new();
    let cap_id = table
        .insert_root(
            memory.object_id(),
            CapKind::Memory,
            PrincipalId::new(7),
            CapPerms::READ,
            memory.generation(),
            0,
        )
        .map_err(|_| ObjectRegistryError::ObjectNotFound)?;
    let cap = table
        .get(cap_id)
        .map_err(|_| ObjectRegistryError::ObjectNotFound)?;

    let resolved = registry.resolve_capability(cap, CapPerms::READ)?;

    assert_eq!(resolved, memory);
    assert_eq!(
        registry.resolve_capability(cap, CapPerms::WRITE),
        Err(ObjectRegistryError::MissingPermission)
    );

    Ok(())
}

#[test]
fn object_capability_resolution_rejects_wrong_kind_and_stale_generation()
-> Result<(), ObjectRegistryError> {
    let mut registry = ObjectRegistry::<2>::new();
    let endpoint = registry.create(ObjectCreate::endpoint(object_id(2), owner(0)))?;
    let mut table = CapabilityTable::<3>::new();
    let wrong_kind = table
        .insert_root(
            endpoint.object_id(),
            CapKind::Queue,
            PrincipalId::new(7),
            CapPerms::READ,
            endpoint.generation(),
            0,
        )
        .map_err(|_| ObjectRegistryError::ObjectNotFound)?;
    let stale = table
        .insert_root(
            endpoint.object_id(),
            CapKind::Endpoint,
            PrincipalId::new(7),
            CapPerms::READ,
            endpoint.generation() + 1,
            0,
        )
        .map_err(|_| ObjectRegistryError::ObjectNotFound)?;

    assert_eq!(
        registry.resolve_capability(
            table
                .get(wrong_kind)
                .map_err(|_| ObjectRegistryError::ObjectNotFound)?,
            CapPerms::READ
        ),
        Err(ObjectRegistryError::WrongCapabilityKind)
    );
    assert_eq!(
        registry.resolve_capability(
            table
                .get(stale)
                .map_err(|_| ObjectRegistryError::ObjectNotFound)?,
            CapPerms::READ
        ),
        Err(ObjectRegistryError::StaleObjectGeneration)
    );

    Ok(())
}

#[test]
fn object_capability_resolution_rejects_recycled_stale_generation()
-> Result<(), ObjectRegistryError> {
    let mut registry = ObjectRegistry::<1>::new();
    let first = registry.create(ObjectCreate::endpoint(object_id(2), owner(0)))?;
    let mut table = CapabilityTable::<1>::new();
    let stale = table
        .insert_root(
            first.object_id(),
            CapKind::Endpoint,
            PrincipalId::new(7),
            CapPerms::READ,
            first.generation(),
            0,
        )
        .map_err(|_| ObjectRegistryError::ObjectNotFound)?;

    registry.delete(first.object_id())?;
    let second = registry.create(ObjectCreate::endpoint(first.object_id(), owner(0)))?;

    assert_eq!(second.generation(), first.generation() + 1);
    assert_eq!(
        registry.resolve_capability(
            table
                .get(stale)
                .map_err(|_| ObjectRegistryError::ObjectNotFound)?,
            CapPerms::READ
        ),
        Err(ObjectRegistryError::StaleObjectGeneration)
    );

    Ok(())
}

#[test]
fn object_record_debug_redacts_object_identifier() -> Result<(), ObjectRegistryError> {
    let mut registry = ObjectRegistry::<1>::new();
    let record = registry.create(ObjectCreate::memory(object_id(0xfeed_cafe), owner(0)))?;
    let rendered = format!("{record:?}");

    assert!(rendered.contains("<redacted>"));
    assert!(!rendered.contains("feed"));
    assert!(!rendered.contains("ObjectId"));

    Ok(())
}
