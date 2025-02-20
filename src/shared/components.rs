use bevy::ecs::query::QueryData;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::utils::Duration;
use lightyear::inputs::leafwing::input_buffer::InputBuffer;
use server::ControlledEntities;
use std::hash::{Hash, Hasher};

use avian3d::prelude::*;
use bevy::prelude::TransformSystem::TransformPropagate;
use leafwing_input_manager::prelude::ActionState;
use lightyear::shared::replication::components::Controlled;
use tracing::Level;

use crate::common::shared::FIXED_TIMESTEP_HZ;
use lightyear::prelude::client::*;
use lightyear::prelude::TickManager;
use lightyear::prelude::*;

use crate::protocol::*;

use super::*;

#[derive(Bundle)]
pub(crate) struct CharacterPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
    lock_axes: LockedAxes,
    friction: Friction,
}

impl Default for CharacterPhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_HEIGHT,
            ),
            rigid_body: RigidBody::Dynamic,
            external_force: ExternalForce::ZERO.with_persistence(false),
            external_impulse: ExternalImpulse::ZERO.with_persistence(false),
            lock_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            friction: Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
        }
    }
}

#[derive(Bundle)]
pub(crate) struct FloorPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
}

impl Default for FloorPhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(
                FLOOR_WIDTH,
                FLOOR_HEIGHT,
                FLOOR_WIDTH,
            ),
            rigid_body: RigidBody::Static,
        }
    }
}

#[derive(Bundle)]
pub(crate) struct BlockPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
}

impl Default for BlockPhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(
                BLOCK_WIDTH,
                BLOCK_HEIGHT,
                BLOCK_WIDTH,
            ),
            rigid_body: RigidBody::Dynamic,
        }
    }
}
