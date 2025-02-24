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

use systems::*;
use crate::quantization::QuantizationPlugin;

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
        app.register_type::<HistoryBuffer<Position>>();
        // Physics

        // Position and Rotation are the primary source of truth so no need to
        // sync changes from Transform to Position.
        // (we are not applying manual updates to Transform)
        app.insert_resource(
            avian3d::sync::SyncConfig {
                transform_to_position: false,
                position_to_transform: true,
                ..default()
            },
        );
        // disable sleeping
        app.insert_resource(
            SleepingThreshold {
                linear: -0.01,
                angular: -0.01,
            },
        );

        // NOTE: does not help
        // app.insert_resource(SolverConfig {
        //     warm_start_coefficient: 0.0,
        //     ..default()
        // });
        app.insert_resource(Gravity(Vec3::ZERO));

        // check the component values right after 'prepare-rollback', which should reset all component
        // values to be equal to the server
        app.add_systems(PreUpdate, after_physics_log.after(PredictionSet::PrepareRollback).before(PredictionSet::Rollback).run_if(is_in_rollback));
        app.add_systems(FixedUpdate, apply_force_to_cube_system);
        app.add_systems(
            FixedPostUpdate,
            after_physics_log.after(PhysicsSet::StepSimulation),
        );

        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsInterpolationPlugin>(),
            // disable Sleeping plugin as it can mess up physics rollbacks
            // TODO: disabling sleeping plugin causes the player to fall through the floor
            // .disable::<SleepingPlugin>(),
        );

        // app.add_plugins(QuantizationPlugin::new(FixedPostUpdate));


        app.add_systems(FixedPostUpdate, after_physics_log_player);
        app.add_systems(PostProcessCollisions, correct_small_differences);
    }
}
