// Adapted from https://github.com/Laumania/Unity3d-PhysicsGun

use crate::{
  player::{controller::CharacterController, raycast::ViewInfo, spawn::Player},
  prelude::*,
  shaders::{AttachShaderEvent, DetachShaderEvent},
};
use bevy::{
  input::mouse::{MouseMotion, MouseWheel},
  render::{
    pipeline::{CullMode, PipelineDescriptor, PrimitiveState},
    shader::ShaderStages,
  },
};
use bevy_rapier3d::{
  na::UnitQuaternion,
  physics::RigidBodyComponentsQuery,
  prelude::*,
  rapier::{
    dynamics::{BodyStatus, MassProperties, RigidBodyHandle},
    na::Vector3,
  },
};

struct ToolStateInner {
  held_body: Entity,
  distance: f32,
  hit_offset: Vector3<f32>,
  rotation_difference: UnitQuaternion<f32>,
  accumulated_rotation: UnitQuaternion<f32>,
}

#[derive(Default)]
struct ToolState(Option<ToolStateInner>);

struct CachedMassProperties(RigidBodyMassProps);

fn tool_system(
  mut commands: Commands,
  mouse_input: Res<Input<MouseButton>>,
  player: Res<Player>,
  mut tool_state: ResMut<ToolState>,
  transform_query: Query<&GlobalTransform>,
  cached_mass_properties: Query<&CachedMassProperties>,
  mut attach_shader: ResMut<Events<AttachShaderEvent>>,
  mut detach_shader: ResMut<Events<DetachShaderEvent>>,
  outline_shader: Res<OutlineShader>,
  view_info: ResMut<ViewInfo>,
  mut body_query: Query<(
    &mut RigidBodyMassProps,
    &mut RigidBodyVelocity,
    &RigidBodyType,
    &RigidBodyPosition,
  )>,
) {
  match tool_state.0.as_ref() {
    Some(inner) => {
      let entity = inner.held_body;
      let (mut mass_props, mut velocity, _, _) = body_query.get_mut(entity).unwrap();
      let reset = if !mouse_input.pressed(MouseButton::Left) {
        true
      } else if mouse_input.just_pressed(MouseButton::Right) {
        // Save mass properties to unfreeze later
        commands
          .entity(entity)
          .insert(CachedMassProperties(mass_props.clone()));

        // Freeze entity
        mass_props.local_mprops.set_mass(0., true);
        velocity.linvel = Vector3::zeros();
        velocity.angvel = Vector3::zeros();

        true
      } else {
        false
      };

      if reset {
        #[cfg(not(target_arch = "wasm32"))]
        detach_shader.send(DetachShaderEvent {
          entity,
          pipeline: outline_shader.0.clone(),
        });
        tool_state.0 = None;
      }
    }

    None => {
      if !mouse_input.just_pressed(MouseButton::Left) {
        return;
      }

      if let Some(hit) = &view_info.hit {
        let (mut mass_props, _, body_status, position) = body_query.get_mut(hit.entity).unwrap();
        if *body_status == BodyStatus::Dynamic {
          if let Ok(mass_properties) = cached_mass_properties.get(hit.entity) {
            *mass_props = mass_properties.0;
          }

          #[cfg(not(target_arch = "wasm32"))]
          attach_shader.send(AttachShaderEvent {
            entity: hit.entity,
            pipeline: outline_shader.0.clone(),
          });

          let hit_point = view_info.ray.point_at(hit.intersection.toi);
          let player_transform = transform_query.get(player.camera).unwrap();
          let obj_transform = position.position;

          tool_state.0 = Some(ToolStateInner {
            held_body: hit.entity,
            distance: hit.intersection.toi,
            hit_offset: obj_transform.translation.vector - hit_point.coords,
            rotation_difference: player_transform.rotation.to_na_unit_quat().inverse()
              * obj_transform.rotation,
            accumulated_rotation: UnitQuaternion::identity(),
          });
        }
      }
    }
  }
}

const FORCE_MULTIPLIER: f32 = 5.;
const MOUSE_WHEEL_MULTIPLIER: f32 = 3.;
const DISTANCE_MIN: f32 = 3.;

// fn move_system(
//   time: Res<Time>,
//   keyboard_input: Res<Input<KeyCode>>,
//   mut mouse_wheel_reader: EventReader<MouseWheel>,
//   mut mouse_motion_reader: EventReader<MouseMotion>,
//   mut tool_state: ResMut<ToolState>,
//   mut bodies: ResMut<RigidBodySet>,
//   player: Res<Player>,
//   transform_query: Query<&GlobalTransform>,
//   view_info: ResMut<ViewInfo>,
//   controller: Res<CharacterController>,
// ) {
//   if let Some(inner) = tool_state.0.as_mut() {
//     // Change distance from player based on mouse wheel
//     for event in mouse_wheel_reader.iter() {
//       inner.distance =
//         (inner.distance + event.y.signum() * MOUSE_WHEEL_MULTIPLIER * -1.).max(DISTANCE_MIN);
//     }

//     let body = bodies.get_mut(inner.held_body).unwrap();
//     let target_pos = view_info.ray.point_at(inner.distance).coords + inner.hit_offset;
//     let current_pos = body.position().translation.vector;
//     let force = (target_pos - current_pos) / time.delta_seconds() * body.mass() * FORCE_MULTIPLIER;

//     let player_transform = transform_query.get(player.camera).unwrap();
//     let player_rotation = player_transform.rotation.to_na_unit_quat();

//     if keyboard_input.pressed(controller.input_map.key_rotate_toolgun) {
//       for event in mouse_motion_reader.iter() {
//         let delta = event.delta;
//         let snap_mode = keyboard_input.pressed(controller.input_map.key_lock_rotation);

//         // After testing, if snap rotation accumulates as normal rotation, then feels too fast
//         let multiplier = if snap_mode { 0.005 } else { 0.01 };
//         let dx = delta.x as f32 * multiplier;
//         let dy = delta.y as f32 * multiplier;

//         let rx = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), dx);
//         let ry = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), dy);

//         let round_to_nearest = |n: f32, r: f32| (n / r).round() * r;
//         let snap = |q: UnitQuaternion<f32>| {
//           let (r, p, y) = q.euler_angles();
//           let deg = std::f32::consts::PI / 4.;
//           UnitQuaternion::from_euler_angles(
//             round_to_nearest(r, deg),
//             round_to_nearest(p, deg),
//             round_to_nearest(y, deg),
//           )
//         };

//         if snap_mode {
//           inner.accumulated_rotation = rx * ry * inner.accumulated_rotation;
//           inner.rotation_difference = snap(inner.rotation_difference);
//           let new_diff = snap(inner.accumulated_rotation * inner.rotation_difference);
//           if inner.rotation_difference.angle_to(&new_diff) > 0.01 {
//             inner.rotation_difference = new_diff;
//             inner.accumulated_rotation = UnitQuaternion::identity();
//           }
//         } else {
//           inner.rotation_difference = rx * ry * inner.rotation_difference;
//         }
//       }
//     }

//     let desired_rotation = player_rotation * inner.rotation_difference;
//     inner.rotation_difference = player_rotation.inverse() * desired_rotation;

//     let current_rotation = body.position().rotation;
//     let rotation_delta = current_rotation.rotation_to(&desired_rotation);
//     let torque =
//       rotation_delta.scaled_axis() / time.delta_seconds() * body.mass() * FORCE_MULTIPLIER;

//     body.set_linvel(Vector3::zeros(), true);
//     body.set_angvel(Vector3::zeros(), true);
//     body.apply_force(force, true);
//     body.apply_torque(torque, true);
//   }
// }

#[derive(Default)]
struct OutlineShader(Handle<PipelineDescriptor>);

fn tool_assets(
  mut pipelines: ResMut<Assets<PipelineDescriptor>>,
  mut outline_shader: ResMut<OutlineShader>,
  asset_server: Res<AssetServer>,
) {
  outline_shader.0 = pipelines.add(PipelineDescriptor {
    primitive: PrimitiveState {
      cull_mode: CullMode::Front,
      ..Default::default()
    },
    ..PipelineDescriptor::default_config(ShaderStages {
      vertex: asset_server.load("shaders/silhouette.vert"),
      fragment: Some(asset_server.load("shaders/silhouette.frag")),
    })
  });
}

pub struct ToolPlugin;
impl Plugin for ToolPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<OutlineShader>()
      .init_resource::<ToolState>()
      .add_system(tool_system.system())
      // .add_system(move_system.system())
      .add_startup_system(tool_assets.system());
  }
}
