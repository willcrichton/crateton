use crate::{
  assets::{AssetState, ASSET_STAGE},
  map::{MapAssets, SpawnModelEvent},
  player::controller::CharacterController,
  prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_world_visualizer::WorldVisualizerParams;
use std::collections::HashMap;

#[derive(Default)]
struct InternedTextures {
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
  mut spawn_model_events: ResMut<Events<SpawnModelEvent>>,
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
      for model_name in map_assets.thumbnails.keys() {
        if let Some(texture_id) = interned_textures.get_egui_id(model_name) {
          let thumbnail = ui.add(egui::widgets::ImageButton::new(
            egui::TextureId::User(texture_id),
            [64.0, 64.0],
          ));

          if thumbnail.clicked {
            spawn_model_events.send(SpawnModelEvent {
              model_name: model_name.clone(),
            })
          }
        }
      }
    });
  }
}

fn toggle_world_visualizer(
  mut params: ResMut<WorldVisualizerParams>,
  keyboard_input: Res<Input<KeyCode>>,
  character_controller: Res<CharacterController>,
  mut windows: ResMut<Windows>,
) {
  let window = windows.get_primary_mut().unwrap();

  let key = character_controller.input_map.key_toggle_world_visualizer;
  params.show = keyboard_input.pressed(key);
  if keyboard_input.just_pressed(key) || keyboard_input.just_released(key) {
    window.set_cursor_lock_mode(!params.show);
    window.set_cursor_visibility(params.show);
    // NOTE: potential inconsistency if pressing tab and alt together
  }
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<InternedTextures>()
      .add_system(ui_system.system())
      .add_system(toggle_world_visualizer.system())
      .add_plugin(EguiPlugin)
      .on_state_enter(ASSET_STAGE, AssetState::Finished, load_assets.system());
  }
}
