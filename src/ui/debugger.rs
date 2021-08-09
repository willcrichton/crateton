use crate::{player::controller::CharacterController, prelude::*};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::{
  world_inspector::WorldUIContext, Context, Inspectable, InspectableRegistry, WorldInspectorParams,
};

use super::{UiLock, UiWindowManager};

fn debugger_system(world: &mut World) {
  let world_ptr = world as *mut _;
  let (mut ui_lock, keyboard_input, character_controller, mut ui_window_manager) = unsafe {
    (
      world
        .get_resource_unchecked_mut::<DebuggerUiLock>()
        .unwrap(),
      world.get_resource::<Input<KeyCode>>().unwrap(),
      world.get_resource::<CharacterController>().unwrap(),
      world
        .get_resource_unchecked_mut::<UiWindowManager>()
        .unwrap(),
    )
  };

  let key = character_controller.input_map.key_toggle_world_visualizer;

  if keyboard_input.just_pressed(key) {
    ui_lock.0 = ui_window_manager.try_show();
  } else if keyboard_input.just_released(key) && ui_lock.0.is_some() {
    let lock = ui_lock.0.take().unwrap();
    ui_window_manager.unshow(lock);
  }

  if ui_lock.0.is_some() {
    let egui_context = world.get_resource::<EguiContext>().unwrap();
    let ctx = egui_context.ctx();
    egui::Window::new("Debugger").scroll(true).show(ctx, |ui| {
      let world: &mut World = unsafe { &mut *world_ptr };
      let mut ui_context = WorldUIContext::new(world, Some(ctx));
      ui_context.world_ui::<()>(ui, &WorldInspectorParams::default());
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
      .add_system(debugger_system.exclusive_system());
  }
}
