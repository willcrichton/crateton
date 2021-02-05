use crate::prelude::*;
use bevy_egui::EguiPlugin;
use std::collections::HashMap;

mod debugger;
mod spawnmenu;
mod terminal;

#[derive(Default)]
pub struct InternedTextures {
  textures: HashMap<String, u64>,
}

impl InternedTextures {
  pub fn get_egui_id(&self, name: &str) -> Option<u64> {
    self.textures.get(name).cloned()
  }

  pub fn add_texture(&mut self, name: String) -> u64 {
    let n = self.textures.len() as u64;
    self.textures.insert(name, n);
    n
  }

  pub fn null_texture(&self) -> u64 {
    u64::MAX
  }
}

#[derive(Default)]
pub struct UiWindowManager {
  showing: isize,
}

impl UiWindowManager {
  pub fn set_showing(&mut self, showing: bool) {
    self.showing += if showing { 1 } else { -1 };
  }

  pub fn is_showing(&self) -> bool {
    self.showing > 0
  }
}

fn ui_window_system(manager: Res<UiWindowManager>, mut windows: ResMut<Windows>) {
  let window = windows.get_primary_mut().unwrap();
  let showing = manager.is_showing();
  window.set_cursor_lock_mode(!showing);
  window.set_cursor_visibility(showing);
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_plugin(EguiPlugin)
      .init_resource::<InternedTextures>()
      .init_resource::<UiWindowManager>()
      .add_system(ui_window_system.system())
      .add_plugin(debugger::DebuggerPlugin)
      .add_plugin(spawnmenu::SpawnmenuPlugin)
      .add_plugin(terminal::TerminalPlugin);
  }
}
