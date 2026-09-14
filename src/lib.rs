use bevy::prelude::*;

/// The entity that this entity depends on.
#[derive(Component, Debug)]
#[relationship(relationship_target = DependedOnBy)]
struct DependsOn(Entity);

/// All entities that depend on this entity.
#[derive(Component, Debug)]
#[relationship_target(relationship = DependsOn)]
struct DependedOnBy(Vec<Entity>);
