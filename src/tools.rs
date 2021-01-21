// Adapted from https://github.com/Laumania/Unity3d-PhysicsGun

use crate::{
  assets::AssetRegistry,
  player::{raycast::ViewInfo, spawn::Player},
  prelude::*,
  shaders::ShaderEvents,
};
use bevy::{
  input::mouse::MouseWheel,
  render::{
    pipeline::{CullMode, PipelineDescriptor, RasterizationStateDescriptor},
    shader::ShaderStages,
  },
};
use bevy_rapier3d::{na::UnitQuaternion, rapier::{dynamics::{BodyStatus, MassProperties, RigidBodyHandle, RigidBodySet}, geometry::ColliderSet, na::Vector3}};

struct ToolStateInner {
  held_body: RigidBodyHandle,
  distance: f32,
  hit_offset: Vector3<f32>,
  rotation_difference: UnitQuaternion<f32>,
}

#[derive(Default)]
struct ToolState(Option<ToolStateInner>);

struct CachedMassProperties(MassProperties);

fn tool_system(
  commands: &mut Commands,
  mouse_input: Res<Input<MouseButton>>,
  player: Res<Player>,
  mut tool_state: ResMut<ToolState>,
  transform_query: Query<&GlobalTransform>,
  cached_mass_properties: Query<&CachedMassProperties>,
  mut shader_events: ResMut<ShaderEvents>,
  outline_shader: Res<OutlineShader>,
  colliders: Res<ColliderSet>,
  mut bodies: ResMut<RigidBodySet>,
  view_info: ResMut<ViewInfo>,
) {
  match tool_state.0.as_ref() {
    Some(inner) => {
      let body = bodies.get_mut(inner.held_body).unwrap();
      let reset = if !mouse_input.pressed(MouseButton::Left) {
        true
      } else if mouse_input.just_pressed(MouseButton::Right) {
        let mass_properties = body.mass_properties().clone();

        // Save mass properties to unfreeze later
        commands.insert_one(body.entity(), CachedMassProperties(mass_properties.clone()));

        // Freeze entity
        body.set_mass_properties(
          MassProperties::new(mass_properties.local_com, 0., Vector3::zeros()),
          false,
        );
        body.set_angvel(Vector3::zeros(), false);
        body.set_linvel(Vector3::zeros(), false);

        true
      } else {
        false
      };

      if reset {
        #[cfg(not(target_arch = "wasm32"))]
        shader_events.detach_shader(body.entity(), outline_shader.0.clone());
        tool_state.0 = None;
      }
    }

    None => {
      if !mouse_input.just_pressed(MouseButton::Left) {
        return;
      }

      if let Some(hit) = &view_info.hit {
        let body_handle = colliders.get(hit.collider_handle).unwrap().parent();
        let body = bodies.get_mut(body_handle).unwrap();
        if body.body_status == BodyStatus::Dynamic {
          if let Ok(mass_properties) = cached_mass_properties.get(body.entity()) {
            body.set_mass_properties(mass_properties.0, false);
          }

          #[cfg(not(target_arch = "wasm32"))]
          shader_events.attach_shader(body.entity(), outline_shader.0.clone());

          let hit_point = view_info.ray.point_at(hit.intersection.toi);
          let player_transform = transform_query.get(player.camera).unwrap();
          let obj_transform = body.position();

          tool_state.0 = Some(ToolStateInner {
            held_body: body_handle,
            distance: hit.intersection.toi,
            hit_offset: obj_transform.translation.vector - hit_point.coords,
            rotation_difference: player_transform.rotation.to_na_unit_quat().inverse()
              * obj_transform.rotation,
          });
        }
      }
    }
  }
}

const FORCE_MULTIPLIER: f32 = 20.;
const MOUSE_WHEEL_MULTIPLIER: f32 = 3.;
const DISTANCE_MIN: f32 = 3.;

fn move_system(
  time: Res<Time>,
  mut mouse_wheel_reader: EventReader<MouseWheel>,
  mut tool_state: ResMut<ToolState>,
  mut bodies: ResMut<RigidBodySet>,
  player: Res<Player>,
  transform_query: Query<&GlobalTransform>,
  view_info: ResMut<ViewInfo>,
) {
  if let Some(inner) = tool_state.0.as_mut() {
    // Change distance from player based on mouse wheel
    for event in mouse_wheel_reader.iter() {
      inner.distance =
        (inner.distance + event.y.signum() * MOUSE_WHEEL_MULTIPLIER * -1.).max(DISTANCE_MIN);
    }

    let body = bodies.get_mut(inner.held_body).unwrap();
    let target_pos = view_info.ray.point_at(inner.distance).coords + inner.hit_offset;
    let current_pos = body.position().translation.vector;
    let force = (target_pos - current_pos) / time.delta_seconds() * FORCE_MULTIPLIER;

    let player_transform = transform_query.get(player.camera).unwrap();
    let player_rotation = player_transform.rotation.to_na_unit_quat();
    // TODO: allow player to change rotation with E
    let desired_rotation = player_rotation * inner.rotation_difference;
    inner.rotation_difference = player_rotation.inverse() * desired_rotation;

    let current_rotation = body.position().rotation;
    let rotation_delta = current_rotation.rotation_to(&desired_rotation);
    let torque = rotation_delta.scaled_axis() / time.delta_seconds() * FORCE_MULTIPLIER;

    body.set_linvel(Vector3::zeros(), true);
    body.set_angvel(Vector3::zeros(), true);
    body.apply_force(force, true);
    body.apply_torque(torque, true);
  }
}

#[derive(Default)]
struct OutlineShader(Handle<PipelineDescriptor>);

fn tool_assets(
  mut pipelines: ResMut<Assets<PipelineDescriptor>>,
  mut outline_shader: ResMut<OutlineShader>,
  mut asset_registry: ResMut<AssetRegistry>,
  asset_server: Res<AssetServer>,
) {
  outline_shader.0 = pipelines.add(PipelineDescriptor {
    rasterization_state: Some(RasterizationStateDescriptor {
      cull_mode: CullMode::Front,
      ..Default::default()
    }),
    ..PipelineDescriptor::default_config(ShaderStages {
      vertex: asset_registry.register_shader(&asset_server, "shaders/silhouette.vert"),
      fragment: Some(asset_registry.register_shader(&asset_server, "shaders/silhouette.frag")),
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
      .add_system(move_system.system())
      .add_startup_system(tool_assets.system());
  }
}
