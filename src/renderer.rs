use crate::protocol::Player;
use crate::{
    protocol::{BlockMarker, Bullet, CharacterMarker, ColorComponent, FloorMarker},
    shared::*,
};
use avian3d::prelude::*;
use bevy::color::palettes::css;
use bevy::color::palettes::css::{RED, WHITE, WHITE_SMOKE};
use bevy::prelude::*;
use bevy_third_person_camera::{ThirdPersonCamera, ThirdPersonCameraPlugin, Zoom};
use lightyear::prelude::server::{Identity, ReplicationTarget};
use lightyear::{
    client::prediction::diagnostics::PredictionDiagnosticsPlugin,
    prelude::{client::*, *},
    transport::io::IoDiagnosticsPlugin,
};

pub struct ExampleRendererPlugin;

impl Plugin for ExampleRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup, init,
        );
        app.add_systems(
            Update,
            (
                add_character_cosmetics,
                add_floor_cosmetics,
                add_block_cosmetics,
                add_bullet_cosmetics,
                draw_characters,
                print_corrections,
            ),
        );

        // Set up visual interp plugins for Transform. Transform is updated in FixedUpdate
        // by the physics plugin so we make sure that in PostUpdate we interpolate it
        app.add_plugins(VisualInterpolationPlugin::<Transform>::default());
        app.add_plugins(VisualInterpolationPlugin::<Position>::default());

        // Observers that add VisualInterpolationStatus components to entities
        // which receive a Position and are predicted
        // app.add_observer(add_visual_interpolation_components);
    }
}

fn init(mut commands: Commands) {
    commands.spawn(
        (
            Camera3d::default(),
            Transform::from_xyz(
                0.0, 4.5, -9.0,
            )
            .looking_at(
                Vec3::ZERO,
                Dir3::Y,
            ),
            ThirdPersonCamera {
                zoom: Zoom::new(
                    5.0, 30.0,
                ),
                cursor_lock_active: false,
                cursor_lock_key: KeyCode::KeyL,
                ..default()
            },
        ),
    );

    commands.spawn(
        (
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(
                4.0, 8.0, 4.0,
            ),
        ),
    );
}

/// Add the VisualInterpolateStatus::<Transform> component to non-floor entities with
/// component `Position`. Floors don't need to be visually interpolated because we
/// don't expect them to move.
///
/// We query Without<Confirmed> instead of With<Predicted> so that the server's
/// gui will also get some visual interpolation. But we're usually just
/// concerned that the client's Predicted entities get the interpolation
/// treatment.
fn add_visual_interpolation_components(
    // We use Position because it's added by avian later, and when it's added
    // we know that Predicted is already present on the entity
    trigger: Trigger<OnAdd, Position>,
    query: Query<
        Entity,
        (
            With<Predicted>,
            Without<FloorMarker>,
        ),
    >,
    mut commands: Commands,
) {
    if !query.contains(trigger.entity()) {
        return;
    }
    commands.entity(trigger.entity()).insert(
        VisualInterpolateStatus::<Transform> {
            // We must trigger change detection on visual interpolation
            // to make sure that child entities (sprites, meshes, text)
            // are also interpolated
            trigger_change_detection: true,
            ..default()
        },
    );
}

/// Add components to characters that impact how they are rendered. We only
/// want to see the predicted character and not the confirmed character.
fn add_character_cosmetics(
    mut commands: Commands,
    character_query: Query<
        (
            Entity,
            &ColorComponent,
        ),
        (
            Or<(
                Added<Predicted>,
                Added<ReplicationTarget>,
            )>,
            With<CharacterMarker>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, color) in &character_query {
        info!(
            ?entity,
            "Adding cosmetics to character {:?}", entity
        );
        commands.entity(entity).insert(
            (
                Mesh3d(
                    meshes.add(
                        Capsule3d::new(
                            CHARACTER_CAPSULE_RADIUS,
                            CHARACTER_CAPSULE_HEIGHT,
                        ),
                    ),
                ),
                MeshMaterial3d(materials.add(color.0)),
            ),
        );
    }
}

/// Add components to floors that impact how they are rendered. We want to see
/// the replicated floor instead of predicted floors because predicted floors
/// do not exist since floors aren't predicted.
fn add_floor_cosmetics(
    mut commands: Commands,
    floor_query: Query<Entity, Added<FloorMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &floor_query {
        info!(
            ?entity,
            "Adding cosmetics to floor {:?}", entity
        );
        commands.entity(entity).insert(
            (
                Mesh3d(
                    meshes.add(
                        Cuboid::new(
                            FLOOR_WIDTH,
                            FLOOR_HEIGHT,
                            FLOOR_WIDTH,
                        ),
                    ),
                ),
                MeshMaterial3d(
                    materials.add(
                        Color::srgb(
                            1.0, 1.0, 1.0,
                        ),
                    ),
                ),
            ),
        );
    }
}

/// Add components to blocks that impact how they are rendered. We only want to
/// see the predicted block and not the confirmed block.
fn add_block_cosmetics(
    mut commands: Commands,
    floor_query: Query<
        Entity,
        (
            Or<(
                Added<Predicted>,
                Added<ReplicationTarget>,
            )>,
            With<BlockMarker>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &floor_query {
        info!(
            ?entity,
            "Adding cosmetics to block {:?}", entity
        );
        commands.entity(entity).insert(
            (
                Mesh3d(
                    meshes.add(
                        Cuboid::new(
                            BLOCK_WIDTH,
                            BLOCK_HEIGHT,
                            BLOCK_WIDTH,
                        ),
                    ),
                ),
                MeshMaterial3d(
                    materials.add(
                        Color::srgb(
                            1.0, 0.0, 1.0,
                        ),
                    ),
                ),
            ),
        );
    }
}

fn print_corrections(player_query: Query<(Entity, &Correction<Position>), With<Player>>) {
    for (entity, correction) in player_query.iter() {
        // info!("correction {:?} -> {}", correction, entity);
    }
}

fn draw_characters(character_query: Query<(&Transform, Has<Predicted>), With<Player>>, mut gizmos: Gizmos) {
    for (transform, is_predicted) in character_query.iter() {
       let color = if is_predicted {css::WHITE} else {RED};

        gizmos.primitive_3d(
            &Capsule3d {
                radius: 0.5,
                half_length: 0.25,
            },
            Isometry3d::new(transform.translation, transform.rotation),
            color,
        );
    }
}

fn add_bullet_cosmetics(
    mut commands: Commands,
    bullet_query: Query<
        (
            Entity,
            &Bullet,
        ),
        (
            Or<(
                Added<Predicted>,
                // Added<PreSpawnedPlayerObject>,
                Added<Replicated>,
            )>,
            With<Bullet>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    identity: NetworkIdentity,
) {
    for (entity, bullet) in &bullet_query {
        let idenity_text = if identity.is_server() { "server" } else { "client" };
        info!(
            "Adding cosmetics to bullet {:?} - {}",
            entity, idenity_text
        );
        commands.entity(entity).insert(
            (
                Mesh3d(meshes.add(Sphere::new(bullet.radius))),
                VisualInterpolateStatus::<Transform>::default(),
                MeshMaterial3d(
                    materials.add(
                        Color::srgb(
                            1.0, 0.0, 1.0,
                        ),
                    ),
                ),
            ),
        );
    }
}
