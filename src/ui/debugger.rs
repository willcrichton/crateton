use crate::{player::controller::CharacterController, prelude::*};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::{Context, Inspectable, InspectableRegistry, WorldInspectorParams};

use super::{UiLock, UiWindowManager};

fn debugger_system(world: &mut World, resources: &mut Resources) {
  let mut ui_lock = resources.get_mut::<DebuggerUiLock>().unwrap();
  let keyboard_input = resources.get::<Input<KeyCode>>().unwrap();
  let character_controller = resources.get::<CharacterController>().unwrap();
  let mut ui_window_manager = resources.get_mut::<UiWindowManager>().unwrap();

  let key = character_controller.input_map.key_toggle_world_visualizer;

  if keyboard_input.just_pressed(key) {
    ui_lock.0 = ui_window_manager.try_show();
  } else if keyboard_input.just_released(key) && ui_lock.0.is_some() {
    let lock = ui_lock.0.take().unwrap();
    ui_window_manager.unshow(lock);
  }

  if ui_lock.0.is_some() {
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

#[derive(Default)]
struct DebuggerUiLock(Option<UiLock>);

pub struct DebuggerPlugin;
impl Plugin for DebuggerPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<InspectableRegistry>()
      .init_resource::<DebuggerUiLock>()
      .add_system(debugger_system.system());
  }
}
