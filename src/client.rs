use crate::common::shared::*;
use crate::protocol::*;
use crate::shared::components::*;
use crate::shared::systems::CharacterQuery;
use crate::shared::systems::*;
use avian3d::prelude::*;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_third_person_camera::*;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::client::interpolation::plugin::InterpolationPlugin;
use lightyear::inputs::leafwing::input_buffer::InputBuffer;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use lightyear::shared::replication::components::Controlled;
use lightyear::shared::tick_manager;

pub struct ExampleClientPlugin;

impl Plugin for ExampleClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ThirdPersonCameraPlugin);
        app.configure_sets(PostUpdate, CameraSyncSet.after(PhysicsSet::Sync));

        app.add_systems(PreUpdate, handle_client_connection_system.after(MainSet::Receive));


        app.add_systems(Startup, connect_to_server);
        app.add_systems(
            FixedUpdate,
            (
                // In host-server mode, the server portion is already applying the
                // character actions, and so we don't want to apply the character
                // actions twice.
                handle_character_actions.run_if(not(is_host_server)),
                player_firing.run_if(not(is_in_rollback)),
            )
                .chain(),
        );
        app.add_systems(
            PreUpdate,
            (handle_new_floor, handle_new_block, handle_new_character)
                .after(PredictionSet::Sync)
                .before(PredictionSet::CheckRollback),
        );
    }
}


/// Listen for events to know when the client is connected;
/// - spawn a text entity to display the client id
/// - spawn a client-owned cursor entity that will be replicated to the server
pub(crate) fn handle_client_connection_system(
    mut commands: Commands,
    mut connection_event: EventReader<ConnectEvent>,
) {
    for event in connection_event.read() {
        let client_id = event.client_id();
        info!("Spawning local box");
        // spawn a local cursor which will be replicated to other clients, but remain client-authoritative.
        commands.spawn(
            (
                Name::new("Block"),
                BlockPhysicsBundle::default(),
                BlockMarker,
                Position::new(
                    Vec3::new(
                        -1.0, 3.0, 0.0,
                    ),
                ),
                // LinearVelocity(Vec3::Y * 0.5),
                GravityScale(0.0),
                lightyear::prelude::server::Replicate::default(),
            ),
        );
    }
}

/// Process character actions and apply them to their associated character
/// entity.
fn handle_character_actions(
    time: Res<Time<Fixed>>,
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            &ActionState<CharacterAction>,
            &InputBuffer<CharacterAction>,
            CharacterQuery,
        ),
        With<Predicted>,
    >,
    tick_manager: Res<TickManager>,
    rollback: Option<Res<Rollback>>,
) {
    // Get the current tick. It may be a part of a rollback.
    let tick = rollback
        .as_ref()
        .map(|rb| tick_manager.tick_or_rollback_tick(rb))
        .unwrap_or(tick_manager.tick());

    for (action_state, input_buffer, mut character) in &mut query {
        // Use the current character action if it is.
        if input_buffer.get(tick).is_some() {
            apply_character_action(&time, &spatial_query, action_state, &mut character);
            continue;
        }

        // If the current character action is not real then use the last real
        // character action.
        if let Some((_, prev_action_state)) = input_buffer.get_last_with_tick() {
            apply_character_action(&time, &spatial_query, prev_action_state, &mut character);
        } else {
            // No inputs are in the buffer yet. This can happen during initial
            // connection. Apply the default input (i.e. nothing pressed).
            apply_character_action(&time, &spatial_query, action_state, &mut character);
        }
    }
}

pub(crate) fn connect_to_server(mut commands: Commands) {
    commands.connect_client();
}

/// Add physics to characters that are newly predicted. If the client controls
/// the character then add an input component.
fn handle_new_character(
    connection: Res<ClientConnection>,
    mut commands: Commands,
    mut character_query: Query<(Entity, &ColorComponent, Has<Controlled>), (Added<Predicted>, With<CharacterMarker>)>,
) {
    for (entity, color, is_controlled) in &mut character_query {
        if is_controlled {
            info!("Adding InputMap to controlled and predicted entity {entity:?}");

            let input_map = InputMap::new([(CharacterAction::Jump, KeyCode::Space)])
                .with(CharacterAction::Jump, GamepadButton::South)
                .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
                .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd())
                .with(CharacterAction::Shoot, MouseButton::Right);

            commands.entity(entity).insert((input_map, ThirdPersonCameraTarget));
        } else {
            info!("Remote character replicated to us: {entity:?}");
        }
        let client_id = connection.id();
        info!(?entity, ?client_id, "Adding physics to character");
        commands.entity(entity).insert(CharacterPhysicsBundle::default());
    }
}

/// Add physics to floors that are newly replicated. The query checks for
/// replicated floors instead of predicted floors because predicted floors do
/// not exist since floors aren't predicted.
fn handle_new_floor(
    connection: Res<ClientConnection>,
    mut commands: Commands,
    character_query: Query<Entity, (Added<Replicated>, With<FloorMarker>)>,
) {
    for entity in &character_query {
        info!(?entity, "Adding physics to floor");
        commands.entity(entity).insert(FloorPhysicsBundle::default());
    }
}

/// Add physics to blocks that are newly predicted.
fn handle_new_block(
    connection: Res<ClientConnection>,
    mut commands: Commands,
    character_query: Query<Entity, (Added<Predicted>, With<BlockMarker>)>,
) {
    for entity in &character_query {
        info!(?entity, "Adding physics to block");
        commands
            .entity(entity)
            .insert(BlockPhysicsBundle::default())
            .insert(GravityScale(0.0));
    }
}
