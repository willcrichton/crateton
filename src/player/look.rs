use crate::prelude::*;
use crate::ui::UiWindowManager;

// system that converts delta axis events into pitch and yaw
use super::{
  controller::CharacterController,
  events::{LookDeltaEvent, LookEvent, PitchEvent, YawEvent},
};

use bevy::input::mouse::MouseMotion;

#[derive(Clone, Copy)]
pub struct LookDirection {
  pub forward: Vec3,
  pub right: Vec3,
  pub up: Vec3,
}

impl Default for LookDirection {
  fn default() -> Self {
    Self {
      forward: Vec3::unit_z(),
      right: -Vec3::unit_x(),
      up: Vec3::unit_y(),
    }
  }
}

pub struct LookEntity(pub Entity);

pub fn forward_up(settings: Res<MouseSettings>, mut query: Query<&mut LookDirection>) {
  for mut look in query.iter_mut() {
    let rotation = Quat::from_rotation_ypr(
      settings.yaw_pitch_roll.x,
      settings.yaw_pitch_roll.y,
      settings.yaw_pitch_roll.z,
    );
    look.forward = rotation * -Vec3::unit_z();
    look.right = rotation * Vec3::unit_x();
    look.up = rotation * Vec3::unit_y();
  }
}

pub struct MouseSettings {
  pub sensitivity: f32,
  pub yaw_pitch_roll: Vec3,
}

impl Default for MouseSettings {
  fn default() -> Self {
    Self {
      sensitivity: 0.005,
      yaw_pitch_roll: Vec3::zero(),
    }
  }
}

const PITCH_BOUND: f32 = std::f32::consts::FRAC_PI_2 - 1E-3;

pub fn input_to_look(
  keyboard_input: Res<Input<KeyCode>>,
  mut settings: ResMut<MouseSettings>,
  mut mouse_motion: EventReader<MouseMotion>,
  mut pitch_events: EventWriter<PitchEvent>,
  mut yaw_events: EventWriter<YawEvent>,
  mut look_events: EventWriter<LookEvent>,
  mut look_delta_events: EventWriter<LookDeltaEvent>,
  ui_window_manager: Res<UiWindowManager>,
  controller: Res<CharacterController>,
) {
  if ui_window_manager.is_showing() {
    return;
  }

  // TODO: make this modular
  if keyboard_input.pressed(controller.input_map.key_rotate_toolgun) {
    return;
  }

  let mut delta = Vec2::zero();
  for motion in mouse_motion.iter() {
    // NOTE: -= to invert
    delta -= motion.delta;
  }

  if delta.length_squared() > 1E-6 {
    delta *= settings.sensitivity;
    settings.yaw_pitch_roll += delta.extend(0.0);
    if settings.yaw_pitch_roll.y > PITCH_BOUND {
      settings.yaw_pitch_roll.y = PITCH_BOUND;
    }
    if settings.yaw_pitch_roll.y < -PITCH_BOUND {
      settings.yaw_pitch_roll.y = -PITCH_BOUND;
    }
    look_delta_events.send(LookDeltaEvent(delta.extend(0.0)));
    look_events.send(LookEvent(settings.yaw_pitch_roll));
    pitch_events.send(PitchEvent(settings.yaw_pitch_roll.y));
    yaw_events.send(YawEvent(settings.yaw_pitch_roll.x));
  }
}
