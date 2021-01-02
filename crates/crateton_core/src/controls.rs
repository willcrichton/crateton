//! Adapted from https://github.com/mcpar-land/bevy_fly_camera/blob/master/src/lib.rs

use bevy::{prelude::*, input::mouse::MouseMotion};

trait Vec3Ext {
  fn safe_normalize(&self) -> Vec3;
}

impl Vec3Ext for Vec3 {
  fn safe_normalize(&self) -> Vec3 {
    if self.length() == 0. { self.clone() } else { self.normalize() }
  }
}

pub struct FlyCamera {
  yaw: f32,
  pitch: f32,
  velocity: Vec3,
  sensitivity: f32,
  speed: f32  
}

impl Default for FlyCamera {
  fn default() -> Self {
    Self {
      yaw: 0.,
      pitch: 0.,
      velocity: Vec3::zero(),
      sensitivity: 2.0,
      speed: 1.5
    }
  }
}

fn keyboard_system(
  time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
	mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
	for (mut camera, mut transform) in query.iter_mut() {

    let movement_axis = |plus, minus| {
      if keyboard_input.pressed(plus) { -1.0 }
      else if keyboard_input.pressed(minus) { 1.0 }
      else { 0.0 }
    };

    let axis_fwd = movement_axis(KeyCode::W, KeyCode::S);
    let axis_side = movement_axis(KeyCode::A, KeyCode::D);
    
    let fwd_vec = transform.rotation.mul_vec3(Vec3::unit_z()).normalize();
    let side_vec = Quat::from_rotation_y(90f32.to_radians()).mul_vec3(fwd_vec).safe_normalize();
    let accel = (fwd_vec * axis_fwd + side_vec * axis_side) * camera.speed * time.delta_seconds();
    camera.velocity += accel;

    let friction = camera.velocity.safe_normalize() * -1.0 * time.delta_seconds();
    camera.velocity = if (camera.velocity + friction).signum() != camera.velocity.signum() { 
      Vec3::zero() 
    } else {
      camera.velocity + friction
    };
    
    transform.translation += camera.velocity;
  }
}

#[derive(Default)]
struct State {
	mouse_motion_event_reader: EventReader<MouseMotion>,
}

fn mouse_system(
	time: Res<Time>,
	mut state: ResMut<State>,
	mouse_motion_events: Res<Events<MouseMotion>>,
	mut query: Query<(&mut FlyCamera, &mut Transform)>,    
) {
  let mut delta: Vec2 = Vec2::zero();
	for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
		delta += event.delta;
  }
    
  for (mut camera, mut transform) in query.iter_mut() {
    camera.yaw = camera.yaw - delta.x * camera.sensitivity * time.delta_seconds();
    camera.pitch = (camera.pitch + delta.y * camera.sensitivity * time.delta_seconds()).clamp(-89.9, 89.9);

    transform.rotation = 
      Quat::from_axis_angle(Vec3::unit_y(), camera.yaw.to_radians())
        * Quat::from_axis_angle(-Vec3::unit_x(), camera.pitch.to_radians());
  }
}

pub struct FlyCameraPlugin;

impl Plugin for FlyCameraPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app
		  .init_resource::<State>()
			.add_system(keyboard_system.system())
			.add_system(mouse_system.system());   
	}
}