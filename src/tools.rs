use crate::{math::*, player::{
  raycast::{CastFromEyeDeps, LookDirDeps},
  spawn::Player,
}};
use bevy::prelude::*;
use bevy_rapier3d::rapier::{
  dynamics::{BodyStatus, RigidBodyHandle, RigidBodySet},
  na::Vector3,
};

struct ToolStateInner {
  held_body: RigidBodyHandle,
  distance: f32,
  hit_offset: Vector3<f32>
}

#[derive(Default)]
struct ToolState(Option<ToolStateInner>);

fn tool_system(
  mouse_input: Res<Input<MouseButton>>,
  player: Res<Player>,
  mut bodies: ResMut<RigidBodySet>,
  mut tool_state: ResMut<ToolState>,
  cast_from_eye_deps: CastFromEyeDeps,
  transform_query: Query<&GlobalTransform>
) {
  match tool_state.0.as_ref() {
    Some(_inner) => {
      if !mouse_input.pressed(MouseButton::Left) {
        tool_state.0 = None;
      }
    }
    None => {
      if !mouse_input.just_pressed(MouseButton::Left) {
        return;
      }

      let cast = player.cast_from_eye(&cast_from_eye_deps);
      if let Some(hit) = cast {
        let body_handle = hit.collider.parent();
        let body = bodies.get_mut(body_handle).unwrap();
        let hit_point = hit.ray.point_at(hit.intersection.toi);
        let transform = transform_query.get(hit.entity).unwrap();
        if body.body_status == BodyStatus::Dynamic {
          tool_state.0 = Some(ToolStateInner {
            held_body: body_handle,
            distance: hit.intersection.toi,
            hit_offset: transform.translation.to_vector3() - hit_point.coords
          });
        }
      }
    }
  };
}

fn move_system(
  time: Res<Time>,
  tool_state: ResMut<ToolState>,
  mut bodies: ResMut<RigidBodySet>,
  player: Res<Player>,
  look_dir_deps: LookDirDeps,
) {
  if let Some(inner) = tool_state.0.as_ref() {
    let body = bodies.get_mut(inner.held_body).unwrap();

    let look_dir = player.look_dir(&look_dir_deps);
    let target_pos = 
      look_dir.point_at(inner.distance).coords + inner.hit_offset;

    let current_pos = body.position().translation.vector;

    let multiplier = 20.;
    let force = (target_pos - current_pos) / time.delta_seconds() * multiplier;

    body.set_linvel(Vector3::new(0., 0., 0.), true);
    body.set_angvel(Vector3::new(0., 0., 0.), true);
    body.apply_force(force, true);
  }
}

pub struct ToolPlugin;
impl Plugin for ToolPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<ToolState>()
      .add_system(tool_system.system())
      .add_system(move_system.system());
  }
}
