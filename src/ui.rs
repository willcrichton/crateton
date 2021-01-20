use crate::{
  assets::{AssetState, ASSET_STAGE},
  map::MapAssets,
  player::controller::CharacterController,
  prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use std::collections::HashMap;

#[derive(Default)]
struct InternedTextures {
  textures: HashMap<String, u64>,
}

impl InternedTextures {
  pub fn get_egui_id(&self, name: &str) -> u64 {
    self.textures[name]
  }

  pub fn add_texture(&mut self, name: String) -> u64 {
    let n = self.textures.len() as u64;
    self.textures.insert(name, n);
    n
  }
}

fn load_assets(
  mut egui_context: ResMut<EguiContext>,
  map_assets: Res<MapAssets>,
  mut interned_textures: ResMut<InternedTextures>,
) {
  for (path, thumbnail) in &map_assets.thumbnails {
    let id = interned_textures.add_texture(path.clone());
    egui_context.set_egui_texture(id, thumbnail.as_weak());
  }
}


fn ui_system(
  controller: Res<CharacterController>,
  keyboard_input: Res<Input<KeyCode>>,
  mut windows: ResMut<Windows>,
  mut egui_context: ResMut<EguiContext>,
  map_assets: Res<MapAssets>,
  interned_textures: Res<InternedTextures>,
) {
  let ctx = &mut egui_context.ctx;
  let window = windows.get_primary_mut().unwrap();
  if keyboard_input.just_pressed(controller.input_map.key_show_ui) {
    window.set_cursor_lock_mode(false);
    window.set_cursor_visibility(true);
  } else if keyboard_input.just_released(controller.input_map.key_show_ui) {
    window.set_cursor_lock_mode(true);
    window.set_cursor_visibility(false);
  }

  if keyboard_input.pressed(controller.input_map.key_show_ui) {
    egui::Window::new("Spawn window").show(ctx, |ui| {
      for path in map_assets.thumbnails.keys() {
        let thumbnail = ui.add(egui::widgets::ImageButton::new(
            egui::TextureId::User(interned_textures.get_egui_id(path)),
            [256.0, 256.0],
        ));

        if thumbnail.clicked {
          // TODO: send an event to spawn a model
        }
      }
    });
  }
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<InternedTextures>()
      .add_system(ui_system.system())
      .add_plugin(EguiPlugin)
      .on_state_enter(ASSET_STAGE, AssetState::Finished, load_assets.system());
  }
}
