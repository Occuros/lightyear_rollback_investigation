use crate::common::shared::FIXED_TIMESTEP_HZ;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::utilities::color_from_id;
use lightyear::client::components::{ComponentSyncMode, LerpFn};
use lightyear::client::interpolation::LinearInterpolator;
use lightyear::prelude::client::{self, LeafwingInputConfig};
use lightyear::prelude::server::{Replicate, SyncTarget};
use lightyear::prelude::*;
use lightyear::utils::avian3d::{position, rotation};
use lightyear::utils::bevy::TransformLinearInterpolation;
use tracing_subscriber::util::SubscriberInitExt;

// For prediction, we want everything entity that is predicted to be part of
// the same replication group This will make sure that they will be replicated
// in the same message and that all the entities in the group will always be
// consistent (= on the same tick)
pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);

// Components

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ColorComponent(pub(crate) Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FloorMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlockMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Player {
    pub client_id: ClientId,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Weapon {
    pub cooldown: u16,
    pub last_fire_tick: Tick,
}

// despawns `lifetime` ticks after `origin_tick`
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub(crate) struct Lifetime {
    pub(crate) origin_tick: Tick,
    /// number of ticks to live for
    pub(crate) lifetime: i16,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[require(Position, LinearVelocity, RigidBody(||RigidBody::Dynamic))]
pub struct Bullet {
    pub radius: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum CharacterAction {
    Move,
    Jump,
    Shoot,
}

impl Actionlike for CharacterAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Self::Move => InputControlKind::DualAxis,
            Self::Jump => InputControlKind::Button,
            Self::Shoot => InputControlKind::Button,
        }
    }
}

// Protocol
pub(crate) struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LeafwingInputPlugin::<CharacterAction>::default());

        app.register_component::<ColorComponent>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<Name>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<CharacterMarker>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<FloorMarker>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<BlockMarker>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<Bullet>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once);

        // Fully replicated, but not visual, so no need for lerp/corrections:
        app.register_component::<LinearVelocity>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<AngularVelocity>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<ExternalForce>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<ExternalImpulse>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<Transform>(ChannelDirection::Bidirectional);
            // .add_prediction(ComponentSyncMode::Full);

        app.register_component::<ComputedMass>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);

        // app.register_component::<Weapon>(ChannelDirection::ServerToClient)
        //     .add_prediction(ComponentSyncMode::Full);

        app.register_component::<Player>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Simple);

        // Position and Rotation have a `correction_fn` set, which is used to smear rollback errors
        // over a few frames, just for the rendering part in postudpate.
        //
        // They also set `interpolation_fn` which is used by the VisualInterpolationPlugin to smooth
        // out rendering between fixedupdate ticks.
        app.register_component::<Position>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation_fn(position::lerp);
            // .add_correction_fn(position::lerp);
            // .add_correction_fn(
            //     |start: &Position, other: &Position, t: f32| {
            //         // info!("is correction applied");
            //         // if (start.0 - other.0).length() < 0.01 {
            //         //     start.clone()
            //         // } else {
            //         //     Position::new(
            //         //         start.lerp(
            //         //             other.0, t,
            //         //         ),
            //         //     )
            //         // }
            //         other.clone()
            //     },
            // );

        app.register_component::<Rotation>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation_fn(rotation::lerp);
            // .add_correction_fn(rotation::lerp);

        // do not replicate Transform but make sure to register an interpolation function
        // for it so that we can do visual interpolation
        // (another option would be to replicate transform and not use Position/Rotation at all)
        app.add_interpolation::<Transform>(ComponentSyncMode::None);
        app.add_interpolation_fn::<Transform>(TransformLinearInterpolation::lerp);
    }
}
