use crate::{player::controller::CharacterController, prelude::*};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::{Context, Inspectable, InspectableRegistry, WorldInspectorParams};

use super::UiWindowManager;

fn debugger_system(world: &mut World, resources: &mut Resources) {
  let keyboard_input = resources.get::<Input<KeyCode>>().unwrap();
  let character_controller = resources.get::<CharacterController>().unwrap();
  let mut ui_window_manager = resources.get_mut::<UiWindowManager>().unwrap();

  let key = character_controller.input_map.key_toggle_world_visualizer;
  let show = keyboard_input.pressed(key);
  if keyboard_input.just_pressed(key) {
    if ui_window_manager.is_showing() {
      return;
    } else {
      ui_window_manager.set_showing(true);
    }
  } else if keyboard_input.just_released(key) {
    ui_window_manager.set_showing(false);
  }

  if show {
    let egui_context = resources.get::<EguiContext>().unwrap();
    let ctx = &egui_context.ctx;
    egui::Window::new("Debugger").scroll(true).show(ctx, |ui| {
      world.ui(
        ui,
        WorldInspectorParams {
          cluster_by_archetype: false,
          ..Default::default()
        },
        &Context {
          id: None,
          resources: Some(resources),
          world: None,
        },
      );
    });
  }
}

pub struct DebuggerPlugin;
impl Plugin for DebuggerPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<InspectableRegistry>()
      .add_system(debugger_system.system());
  }
}
