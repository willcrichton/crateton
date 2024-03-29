// Adapted from https://github.com/superdump/bevy_prototype_character_controller/
use bevy::prelude::*;

pub mod controller;
pub mod events;
pub mod input_map;
pub mod look;
pub mod physics;
pub mod raycast;
pub mod spawn;

const PROCESS_INPUT_EVENTS: &str = "process_input_events";
const APPLY_INPUT: &str = "apply_input";
const UPDATE_VELOCITY: &str = "update_velocity";

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_startup_system(spawn::spawn_character.system())
      // .add_system(spawn::update_camera.system())
      //
      // Detect keyboard + mouse events
      .add_event::<events::PitchEvent>()
      .add_event::<events::YawEvent>()
      .add_event::<events::LookEvent>()
      .add_event::<events::LookDeltaEvent>()
      .add_event::<events::TranslationEvent>()
      .add_event::<events::ImpulseEvent>()
      .add_event::<events::ForceEvent>()
      .init_resource::<look::MouseSettings>()
      .init_resource::<controller::CharacterController>()
      .init_resource::<raycast::ViewInfo>()
      .add_system(raycast::compute_view_info.system())
      .add_stage_after(
        CoreStage::PreUpdate,
        PROCESS_INPUT_EVENTS,
        SystemStage::parallel(),
      )
      .add_system_to_stage(PROCESS_INPUT_EVENTS, look::input_to_look.system())
      .add_system_to_stage(PROCESS_INPUT_EVENTS, look::forward_up.system())
      //
      // Turn events into forces on controller
      .add_system_to_stage(PROCESS_INPUT_EVENTS, controller::input_to_events.system())
      .add_system_to_stage(CoreStage::Update, controller::controller_to_yaw.system())
      .add_system_to_stage(CoreStage::Update, controller::controller_to_pitch.system())
      //
      // Apply forces through physics engine
      .add_stage_before(
        PROCESS_INPUT_EVENTS,
        UPDATE_VELOCITY,
        SystemStage::parallel(),
      )
      .add_system_to_stage(UPDATE_VELOCITY, physics::body_to_velocity.system())
      .add_stage_after(PROCESS_INPUT_EVENTS, APPLY_INPUT, SystemStage::parallel())
      .add_system_to_stage(
        APPLY_INPUT,
        physics::controller_to_rapier_dynamic_impulse.system(),
      )
      .add_system_to_stage(APPLY_INPUT, physics::controller_to_fly.system());

    #[cfg(not(target = "wasm32"))]
    app.add_startup_system(spawn::init_hud.system());
  }
}
