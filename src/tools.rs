use crate::player::{look::LookDirection, spawn::Player};
use bevy::prelude::*;
use bevy_rapier3d::{
  physics::RigidBodyHandleComponent,
  rapier::{
    dynamics::{RigidBodyHandle, RigidBodySet, BodyStatus},
    geometry::{ColliderSet, InteractionGroups, Ray},
    na::{Point3, Vector3, Translation3},
    pipeline::QueryPipeline,
  },
};

struct ToolStateInner {
  held_body: RigidBodyHandle,
  distance: f32
}

#[derive(Default)]
struct ToolState(Option<ToolStateInner>);

fn tool_system(
  rapier_pipeline: Res<QueryPipeline>,
  colliders: Res<ColliderSet>,
  mouse_input: Res<Input<MouseButton>>,
  player: Res<Player>,
  global_transform_query: Query<&GlobalTransform>,
  look_direction_query: Query<&LookDirection>,
  body_query: Query<&RigidBodyHandleComponent>,
  mut bodies: ResMut<RigidBodySet>,
  mut tool_state: ResMut<ToolState>,
) {
  match tool_state.0.as_ref() {
    Some(_inner) => {
      if !mouse_input.pressed(MouseButton::Left) {
        println!("resetting");
        tool_state.0 = None;
      }
    }
    None => {
      if !mouse_input.just_pressed(MouseButton::Left) {
        return;
      }

      let look = look_direction_query
        .get_component::<LookDirection>(player.camera)
        .expect("Failed to get LookDirection from Entity");
      let head_pos = global_transform_query.get(player.head).unwrap();
      let origin = head_pos.translation;
      let direction = look.forward;
      let ray = Ray::new(
        Point3::new(origin.x, origin.y, origin.z),
        Vector3::new(direction.x, direction.y, direction.z),
      );
      let player_body = body_query.get(player.body).unwrap().handle();

      rapier_pipeline.interferences_with_ray(
        &colliders,
        &ray,
        f32::MAX,
        InteractionGroups::all(),
        |_handle, collider, intersection| {
          let body_handle = collider.parent();
          if body_handle == player_body {
            return true;
          }

          let body = bodies.get_mut(body_handle).unwrap();
          if body.body_status == BodyStatus::Dynamic {
            println!("Holding: {:?}", body_handle);
            tool_state.0 = Some(ToolStateInner {
              held_body: body_handle,
              distance: intersection.toi
            });
          }          

          false
        },
      );
    }
  };
}

fn move_system(
  time: Res<Time>,
  tool_state: ResMut<ToolState>,
  mut bodies: ResMut<RigidBodySet>,
  player: Res<Player>,
  look_direction_query: Query<&LookDirection>,
  global_transform_query: Query<&GlobalTransform>
) {
  if let Some(inner) = tool_state.0.as_ref() {
    let body = bodies.get_mut(inner.held_body).unwrap();
    let look = look_direction_query
      .get_component::<LookDirection>(player.camera)
      .expect("Failed to get LookDirection from Entity");
    let head_pos = global_transform_query.get(player.head).unwrap();
    let origin = head_pos.translation;
    let direction = look.forward;

    let target_pos = origin + direction * inner.distance;
    let target_pos = Vector3::new(target_pos.x, target_pos.y, target_pos.z);
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
