use avian3d::prelude::*;
use avian3d::prepare::PrepareSet;
use avian3d::sync::SyncSet;
use bevy::{
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};


/// Quantizes all the things
pub struct QuantizationPlugin {
    schedule: Interned<dyn ScheduleLabel>,
}

impl QuantizationPlugin {
    /// Creates a [`QuantizationPlugin`] using the given schedule
    pub fn new(schedule: impl ScheduleLabel) -> Self {
        Self {
            schedule: schedule.intern(),
        }
    }
}

impl Plugin for QuantizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            self.schedule,
            (
                q_position,
                q_linear_velocity,
                q_rotation,
                q_angular_velocity,
            )
                .in_set(PrepareSet::First),
        );
        app.add_systems(
            self.schedule,
            (
                q_position,
                q_linear_velocity,
                q_rotation,
                q_angular_velocity,
            )
                .in_set(SyncSet::First),
        );
    }
}

fn q_position(mut q: Query<&mut Position>) {
    for mut v in q.iter_mut() {
        v.0.quantize();
    }
}

fn q_rotation(mut q: Query<&mut Rotation>) {
    for mut v in q.iter_mut() {
        // #[cfg(feature = "2d")]
        // {
        //     v.cos.quantize();
        //     v.sin.quantize();
        //     // normalize better here?
        //     v.cos = v.cos.clamp(-1.0, 1.0);
        //     v.sin = v.sin.clamp(-1.0, 1.0);
        // }
        // #[cfg(feature = "3d")]
        {
            v.0.quantize();
        }
    }
}

fn q_linear_velocity(mut q: Query<&mut LinearVelocity>) {
    for mut v in q.iter_mut() {
        let before = v.0;
        v.0.quantize();
        if before != v.0 {
            info!("velocity diff: {}", before - v.0)
        }
    }
}

fn q_angular_velocity(mut q: Query<&mut AngularVelocity>) {
    for mut v in q.iter_mut() {
        let before = v.0;
        v.0.quantize();
        // println!("a_angvel, before: {:?}, after: {:?}", before, v.0);
    }
}

// --

/// Scale factor for quantization
/// 1/8192 ≈ 0.00122
pub const QUANTIZE_SCALE: f32 = 8192.0;


/// Trait extension for quantizing `f32`
pub trait QuantizableF32 {
    fn quantize(&mut self);
}

impl QuantizableF32 for f32 {
    fn quantize(&mut self) {
        *self = ((*self * QUANTIZE_SCALE).round() as i32) as f32 / QUANTIZE_SCALE;
    }
}

/// Trait extension for quantizing `Vec2`
pub trait QuantizableVec2 {
    fn quantize(&mut self);
}

impl QuantizableVec2 for Vec2 {
    fn quantize(&mut self) {
        self.x.quantize();
        self.y.quantize();
    }
}

pub trait QuantizableVec3 {
    fn quantize(&mut self);
}

impl QuantizableVec3 for Vec3 {
    fn quantize(&mut self) {
        self.x.quantize();
        self.y.quantize();
        self.z.quantize();
    }
}

/// Trait extension for quantizing `Quat`
pub trait QuantizableQuat {
    fn quantize(&mut self);
}

impl QuantizableQuat for Quat {
    fn quantize(&mut self) {
        self.x.quantize();
        self.y.quantize();
        self.z.quantize();
        self.w.quantize();
        *self = self.normalize();
    }
}
