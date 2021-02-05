use crate::{player::controller::CharacterController, prelude::*};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::{Context, Inspectable, InspectableRegistry, WorldInspectorParams};

fn terminal_system(world: &mut World, resources: &mut Resources) {  
  let keyboard_input = resources.get::<Input<KeyCode>>().unwrap();
  let character_controller = resources.get::<CharacterController>().unwrap();
  let mut windows = resources.get_mut::<Windows>().unwrap();
  let window = windows.get_primary_mut().unwrap();

  let key = character_controller.input_map.key_toggle_world_visualizer;
  let show = keyboard_input.pressed(key);
  if keyboard_input.just_pressed(key) || keyboard_input.just_released(key) {
    window.set_cursor_lock_mode(!show);
    window.set_cursor_visibility(show);
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

pub struct TerminalPlugin;
impl Plugin for TerminalPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<InspectableRegistry>()
      .add_system(terminal_system.system());
  }
}
