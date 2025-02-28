use bevy::ecs::query::QueryData;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::utils::Duration;
use lightyear::client::interpolation::plugin::InterpolationPlugin;
use lightyear::inputs::leafwing::input_buffer::InputBuffer;
use server::ControlledEntities;
use std::hash::{Hash, Hasher};
use avian3d::dynamics::solver::SolverConfig;
use avian3d::prelude::*;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::TransformSystem::TransformPropagate;
use leafwing_input_manager::prelude::ActionState;
use lightyear::shared::replication::components::Controlled;
use tracing::Level;

use crate::common::shared::FIXED_TIMESTEP_HZ;
use lightyear::prelude::client::*;
use lightyear::prelude::TickManager;
use lightyear::prelude::*;

use crate::protocol::*;

pub mod components;
pub mod systems;
pub mod utilities;

use crate::quantization::QuantizationPlugin;
use systems::*;

pub const FLOOR_WIDTH: f32 = 100.0;
pub const FLOOR_HEIGHT: f32 = 1.0;

pub const BLOCK_WIDTH: f32 = 1.0;
pub const BLOCK_HEIGHT: f32 = 1.0;

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.5;
pub const CHARACTER_CAPSULE_HEIGHT: f32 = 0.5;

// Define a custom schedule label
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct QuantizationSchedule;

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.register_type::<Player>();
        app.register_type::<Weapon>();

        app.register_type::<HistoryBuffer<Position>>();
        app.insert_resource(avian3d::sync::SyncConfig {
            transform_to_position: false,
            position_to_transform: true,
            ..default()
        });

        app.insert_resource(
            SleepingThreshold {
                linear: -0.01,
                angular: -0.01,
            },
        );


        app.add_systems(FixedUpdate, apply_force_to_cube_system);

        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsInterpolationPlugin>(),
            // TODO: disabling sleeping plugin causes the player to fall through the floor
            // .disable::<SleepingPlugin>(),
        );

        app.add_systems(PostProcessCollisions, correct_small_differences);
        app.add_systems(FixedPreUpdate, apply_force_to_cube_system);
    }
}
