use crate::prelude::*;
use bevy_egui::{
  egui::{FontDefinitions, FontFamily, TextStyle},
  EguiContext, EguiPlugin,
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
  showing: isize,
}

impl UiWindowManager {
  pub fn set_showing(&mut self, showing: bool) {
    self.showing += if showing { 1 } else { -1 };
    debug_assert!(self.showing >= 0);
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

fn configure_fonts(mut egui_context: ResMut<EguiContext>) {
  let ctx = &mut egui_context.ctx;
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
      .add_startup_system(configure_fonts.system())
      // Individual UI plugins
      .add_plugin(debugger::DebuggerPlugin)
      .add_plugin(spawnmenu::SpawnmenuPlugin)
      .add_plugin(terminal::TerminalPlugin);
  }
}
