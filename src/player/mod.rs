// Adapted from https://github.com/superdump/bevy_prototype_character_controller/
use bevy::prelude::*;
use bevy_rapier3d::{
  na::Vector3,
  rapier::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder},
};

mod controller;
mod events;
mod input_map;
mod look;
mod physics;

const PROCESS_INPUT_EVENTS: &str = "process_input_events";
const APPLY_INPUT: &str = "apply_input";
const UPDATE_VELOCITY: &str = "update_velocity";

fn spawn_character(commands: &mut Commands) {
  let height = 1.0;
  let head_scale = 0.3;
  let body = commands
    .spawn((
      controller::BodyTag,
      controller::CharacterController::default(),
      RigidBodyBuilder::new_dynamic()
        .translation(0., height*2., 0.)
        .principal_angular_inertia(Vector3::zeros(), Vector3::repeat(false)),
      ColliderBuilder::cuboid(0.5, 0.5 * height, 0.5).density(200.),
      Transform::identity(),
      GlobalTransform::identity(),
    ))
    .current_entity()
    .unwrap();

  let yaw = commands
    .spawn((
      controller::YawTag,
      Transform::identity(),
      GlobalTransform::identity(),
    ))
    .current_entity()
    .unwrap();

  let head = commands
    .spawn((
      controller::HeadTag,
      GlobalTransform::identity(),
      Transform::from_matrix(Mat4::from_scale_rotation_translation(
        Vec3::one(),
        Quat::from_rotation_y(0.),
        Vec3::new(0.0, 0.5 * head_scale + height - 1.695, 0.0),
      )),
    ))
    .current_entity()
    .unwrap();

  let camera = commands
    .spawn(Camera3dBundle {
      transform: Transform::from_matrix(Mat4::face_toward(
        Vec3::zero(),
        -Vec3::unit_z(),
        Vec3::unit_y(),
      )),
      ..Default::default()
    })
    .with_bundle((look::LookDirection::default(), controller::CameraTag))
    .current_entity()
    .unwrap();

  commands
    .insert_one(body, look::LookEntity(camera))
    .push_children(body, &[yaw])
    .push_children(yaw, &[head])
    .push_children(head, &[camera]);
}

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_event::<events::PitchEvent>()
      .add_event::<events::YawEvent>()
      .add_event::<events::LookEvent>()
      .add_event::<events::LookDeltaEvent>()
      .add_event::<events::TranslationEvent>()
      .add_event::<events::ImpulseEvent>()
      .add_event::<events::ForceEvent>()
      .init_resource::<events::ControllerEvents>()
      .init_resource::<look::MouseMotionState>()
      .init_resource::<look::MouseSettings>()
      .add_stage_after(
        bevy::app::stage::PRE_UPDATE,
        PROCESS_INPUT_EVENTS,
        SystemStage::parallel(),
      )
      .add_system_to_stage(PROCESS_INPUT_EVENTS, controller::input_to_events.system())
      .add_system_to_stage(PROCESS_INPUT_EVENTS, look::input_to_look.system())
      .add_system_to_stage(PROCESS_INPUT_EVENTS, look::forward_up.system())
      .add_system_to_stage(bevy::app::stage::PRE_UPDATE, physics::create_mass.system())
      .add_stage_before(
        PROCESS_INPUT_EVENTS,
        UPDATE_VELOCITY,
        SystemStage::parallel(),
      )
      .add_system_to_stage(UPDATE_VELOCITY, physics::body_to_velocity.system())
      .add_stage_after(PROCESS_INPUT_EVENTS, APPLY_INPUT, SystemStage::parallel())
      .add_system_to_stage(
        APPLY_INPUT,
        physics::controller_to_rapier_dynamic_force.system(),
      )
      .add_system_to_stage(
        bevy::app::stage::UPDATE,
        controller::controller_to_yaw.system(),
      )
      .add_system_to_stage(
        bevy::app::stage::UPDATE,
        controller::controller_to_pitch.system(),
      )
      .add_startup_system(spawn_character.system());
  }
}
