use crate::prelude::*;
use bevy_egui::{
  egui::{FontDefinitions, FontFamily, TextStyle},
  EguiContext, EguiPlugin, EguiSystem,
};
use std::collections::HashMap;

mod debugger;
mod editor;
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
  showing: bool,
}

pub struct UiLock;

impl UiWindowManager {
  pub fn try_show(&mut self) -> Option<UiLock> {
    if self.showing {
      None
    } else {
      self.showing = true;
      Some(UiLock)
    }
  }

  pub fn unshow(&mut self, _lock: UiLock) {
    self.showing = false;
  }

  pub fn is_showing(&self) -> bool {
    self.showing
  }
}

fn ui_window_system(manager: Res<UiWindowManager>, mut windows: ResMut<Windows>) {
  let window = windows.get_primary_mut().unwrap();
  let showing = manager.is_showing();
  window.set_cursor_lock_mode(!showing);
  window.set_cursor_visibility(showing);
}

fn configure_fonts(mut egui_context: ResMut<EguiContext>, mut done: Local<bool>) {
  if *done {
    return;
  }

  *done = true;
  let ctx = egui_context.ctx();
  let mut fonts = FontDefinitions::default();
  fonts
    .family_and_size
    .insert(TextStyle::Monospace, (FontFamily::Monospace, 14.));
  ctx.set_fonts(fonts);
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      // Common UI utilities
      .add_plugin(EguiPlugin)
      .init_resource::<InternedTextures>()
      .init_resource::<UiWindowManager>()
      .add_system(ui_window_system.system())
      
      .add_system(configure_fonts.system())

      // Individual UI plugins
      .add_plugin(debugger::DebuggerPlugin)
      .add_plugin(spawnmenu::SpawnmenuPlugin)
      .add_plugin(terminal::TerminalPlugin);
      ;
  }
}
