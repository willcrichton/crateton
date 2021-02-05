use crate::prelude::*;
use bevy_egui::EguiPlugin;
use std::collections::HashMap;

mod terminal;
mod spawnmenu;

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

pub struct UiPlugin;
impl Plugin for UiPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_plugin(EguiPlugin)
      .init_resource::<InternedTextures>()
      .add_plugin(terminal::TerminalPlugin)
      .add_plugin(spawnmenu::SpawnmenuPlugin);
  }
}
