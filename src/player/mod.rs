// Adapted from https://github.com/superdump/bevy_prototype_character_controller/
use bevy::prelude::*;

pub mod controller;
pub mod events;
pub mod input_map;
pub mod look;
pub mod physics;
pub mod spawn;

const PROCESS_INPUT_EVENTS: &str = "process_input_events";
const APPLY_INPUT: &str = "apply_input";
const UPDATE_VELOCITY: &str = "update_velocity";

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_startup_system(spawn::spawn_character.system())
      //
      // Detect keyboard + mouse events
      .add_event::<events::PitchEvent>()
      .add_event::<events::YawEvent>()
      .add_event::<events::LookEvent>()
      .add_event::<events::LookDeltaEvent>()
      .add_event::<events::TranslationEvent>()
      .add_event::<events::ImpulseEvent>()
      .add_event::<events::ForceEvent>()
      .init_resource::<look::MouseMotionState>()
      .init_resource::<look::MouseSettings>()
      .add_stage_after(
        bevy::app::stage::PRE_UPDATE,
        PROCESS_INPUT_EVENTS,
        SystemStage::parallel(),
      )
      .add_system_to_stage(PROCESS_INPUT_EVENTS, look::input_to_look.system())
      .add_system_to_stage(PROCESS_INPUT_EVENTS, look::forward_up.system())
      //
      // Turn events into forces on controller
      .init_resource::<events::ControllerEvents>()
      .add_system_to_stage(PROCESS_INPUT_EVENTS, controller::input_to_events.system())
      .add_system_to_stage(
        bevy::app::stage::UPDATE,
        controller::controller_to_yaw.system(),
      )
      .add_system_to_stage(
        bevy::app::stage::UPDATE,
        controller::controller_to_pitch.system(),
      )
      //
      // Apply forces through physics engine
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
      .add_system_to_stage(APPLY_INPUT, physics::controller_to_fly.system());
  }
}
