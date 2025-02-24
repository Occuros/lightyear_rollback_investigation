use bevy::ecs::query::QueryData;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::utils::Duration;
use lightyear::inputs::leafwing::input_buffer::InputBuffer;
use lightyear::prelude::server::ReplicationTarget;
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
use crate::shared::*;

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct CharacterQuery {
    pub external_force: &'static mut ExternalForce,
    pub external_impulse: &'static mut ExternalImpulse,
    pub linear_velocity: &'static LinearVelocity,
    pub mass: &'static ComputedMass,
    pub position: &'static Position,
    pub entity: Entity,
}

pub fn player_firing(
    mut commands: Commands,
    mut player_query: Query<
        (
            &Player,
            &mut Weapon,
            &Transform,
            &LinearVelocity,
            &ActionState<CharacterAction>,
            Has<Controlled>,
        ),
        Or<(
            With<Predicted>,
            With<ReplicationTarget>,
        )>,
    >,
    tick_manager: Res<TickManager>,
    identity: NetworkIdentity,
) {
    if player_query.is_empty() {
        return;
    }

    let current_tick = tick_manager.tick();

    for (player, mut weapon, player_transform, player_velocity, player_action, is_local) in player_query.iter_mut() {
        if !player_action.just_pressed(&CharacterAction::Shoot) {
            continue;
        }

        let fired_since = current_tick - weapon.last_fire_tick;

        if fired_since.abs() <= weapon.cooldown as i16 {

            // cooldown period - can't fire.
            if weapon.last_fire_tick == current_tick {
                // logging because debugging latency edge conditions where
                // inputs arrive on exact frame server replicates to you.
                info!("Can't fire, fired this tick already! {current_tick:?}");
            } else {
                // info!("cooldown. {weapon:?} current_tick = {current_tick:?} wrapped_diff: {wrapped_diff}");
            }
            continue;
        }

        info!(
            "spawning bullet for client {} at {:?} since: {} cool: {} last: {:?}",
            player.client_id, current_tick, fired_since, weapon.cooldown, weapon.last_fire_tick
        );

        weapon.last_fire_tick = current_tick;
        let offset = -player_transform.forward() * 0.3;

        let bullet_origin = player_transform.translation + -player_transform.up() * 0.1 + offset;

        let prespawn = PreSpawnedPlayerObject::default_with_salt(player.client_id.to_bits());

        let bullet_size = 0.1;
        let bullet_entity = commands
            .spawn(
                (
                    // Name::new("Bullet"),
                    Bullet {
                        radius: bullet_size,
                    },
                    Position::new(bullet_origin),
                    // LinearVelocity(
                    //     (-player_transform.forward().as_vec3() + player_transform.up().as_vec3()).normalize() * 10.0,
                    // ),
                    LinearVelocity(
                        (-player_transform.forward().as_vec3()).normalize() * 10.0,
                    ),
                    Collider::sphere(bullet_size),
                    // GravityScale(0.0),
                    // Replicate::default(),
                    prespawn,
                ),
            )
            .id();

        if identity.is_server() {
            let replicate = server::Replicate {
                sync: server::SyncTarget {
                    prediction: NetworkTarget::All,
                    ..Default::default()
                },
                group: REPLICATION_GROUP,
                ..default()
            };
            commands.entity(bullet_entity).insert(replicate);
        }
    }
}

pub(crate) fn after_physics_log_player(
    tick_manager: Res<TickManager>,
    rollback: Option<Res<Rollback>>,
    collisions: Option<Res<Collisions>>,
    blocks: Query<
        (
            Entity,
            &Position,
            &Rotation,
            &LinearVelocity,
            &AngularVelocity,
            // &Correction<Position>
        ),
        (
            With<Player>,
        ),
    >,
) {
    let tick = rollback.as_ref().map_or(
        tick_manager.tick(),
        |r| tick_manager.tick_or_rollback_tick(r.as_ref()),
    );
    // info!(?tick, ?collisions, "collisions");
    let is_rollback = rollback.map_or(
        false,
        |r| r.is_rollback(),
    );

    // info!("tick: {:?} collisions: {:?}", tick, collisions);
    // for (entity, position, rotation, lv, av, correction) in blocks.iter() {
        // info!(
        //     ?is_rollback,
        //     ?tick,
        //     ?entity,
        //     ?position,
        //     ?rotation,
        //     ?lv,
        //     ?av,
        //     "Block after physics update"
        // );
    // }
}

pub fn apply_force_to_cube_system(
    mut block_query: Query<(&Transform, &mut LinearVelocity), (With<BlockMarker>, Without<Confirmed>)>
) {
    for (transform, mut  velocity) in block_query.iter_mut() {
        if transform.translation.y > 5.0 {
            velocity.y = -0.5;
        } else if transform.translation.y < 2.0{
            velocity.y = 0.5;
        }
    }
}


pub(crate) fn after_physics_log(
    tick_manager: Res<TickManager>,
    rollback: Option<Res<Rollback>>,
    // collisions: Option<Res<Collisions>>,
    blocks: Query<
        (
            Entity,
            &Position,
            &Rotation,
            &LinearVelocity,
            &AngularVelocity,
            Option<&Correction<Position>>,
        ),
        (
            Without<Confirmed>,
            With<BlockMarker>,
        ),
    >,
) {
    let tick = rollback.as_ref().map_or(
        tick_manager.tick(),
        |r| tick_manager.tick_or_rollback_tick(r.as_ref()),
    );
    // info!(?tick, ?collisions, "collisions");
    let is_rollback = rollback.map_or(
        false,
        |r| r.is_rollback(),
    );
    for (entity, position, rotation, lv, av, correction) in blocks.iter() {
        warn!(
            ?is_rollback,
            ?tick,
            ?entity,
            ?position,
            ?rotation,
            ?lv,
            ?av,
            ?correction,
            "Block after physics update"
        );
    }
}

/// Apply the character actions `action_state` to the character entity `character`.
pub fn apply_character_action(
    time: &Res<Time<Fixed>>,
    spatial_query: &SpatialQuery,
    action_state: &ActionState<CharacterAction>,
    character: &mut CharacterQueryItem,
) {
    const MAX_SPEED: f32 = 5.0;
    const MAX_ACCELERATION: f32 = 20.0;

    // How much velocity can change in a single tick given the max acceleration.
    let max_velocity_delta_per_tick = MAX_ACCELERATION * time.delta_secs();

    // Handle jumping.
    if action_state.pressed(&CharacterAction::Jump) {
        let ray_cast_origin = character.position.0
            + Vec3::new(
                0.0,
                -CHARACTER_CAPSULE_HEIGHT / 2.0 - CHARACTER_CAPSULE_RADIUS,
                0.0,
            );

        // Only jump if the character is on the ground.
        //
        // Check if we are touching the ground by sending a ray from the bottom
        // of the character downwards.
        if spatial_query
            .cast_ray(
                ray_cast_origin,
                Dir3::NEG_Y,
                0.01,
                true,
                &SpatialQueryFilter::from_excluded_entities([character.entity]),
            )
            .is_some()
        {
            character.external_impulse.apply_impulse(
                Vec3::new(
                    0.0, 5.0, 0.0,
                ),
            );
        }
    }

    // Handle moving.
    let move_dir = action_state.axis_pair(&CharacterAction::Move).clamp_length_max(1.0);
    let move_dir = Vec3::new(
        -move_dir.x,
        0.0,
        move_dir.y,
    );

    // Linear velocity of the character ignoring vertical speed.
    let ground_linear_velocity = Vec3::new(
        character.linear_velocity.x,
        0.0,
        character.linear_velocity.z,
    );

    let desired_ground_linear_velocity = move_dir * MAX_SPEED;

    let new_ground_linear_velocity = ground_linear_velocity.move_towards(
        desired_ground_linear_velocity,
        max_velocity_delta_per_tick,
    );

    // Acceleration required to change the linear velocity from
    // `ground_linear_velocity` to `new_ground_linear_velocity` in the duration
    // of a single tick.
    //
    // There is no need to clamp the acceleration's length to
    // `MAX_ACCELERATION`. The difference between `ground_linear_velocity` and
    // `new_ground_linear_velocity` is never great enough to require more than
    // `MAX_ACCELERATION` in a single tick, This is because
    // `new_ground_linear_velocity` is calculated using
    // `max_velocity_delta_per_tick` which restricts how much the velocity can
    // change in a single tick based on `MAX_ACCELERATION`.
    let required_acceleration = (new_ground_linear_velocity - ground_linear_velocity) / time.delta_secs();

    character
        .external_force
        .apply_force(required_acceleration * character.mass.value());
}


pub fn correct_small_differences(mut collisions: ResMut<Collisions>) {
    // collisions.retain(|collision| {
    //     if collision.total_normal_impulse < 0.1 {
    //         info!("very small impulse {}", collision.total_normal_impulse);
    //         false
    //     } else {
    //         true
    //     }
    // })
}