use crate::{
  models::{ModelInfo, ModelParams, SceneDecomposition, SpawnModelEvent, Thumbnail},
  player::{controller::CharacterController, raycast::ViewInfo},
  prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::world_inspector::{InspectableRegistry, WorldUIContext};
use bevy_rapier3d::na::{Isometry3, Translation3, UnitQuaternion, Vector3};
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

  pub fn null_texture(&self) -> u64 {
    u64::MAX
  }
}

fn load_assets(
  mut egui_context: ResMut<EguiContext>,
  mut interned_textures: ResMut<InternedTextures>,
  query: Query<(&ModelInfo, &Thumbnail), Changed<Thumbnail>>,
) {
  for (model_info, thumbnail) in query.iter() {
    let id = interned_textures.add_texture(model_info.name.clone());
    egui_context.set_egui_texture(id, thumbnail.0.as_weak());
  }
}

fn spawn_ui_system(
  controller: Res<CharacterController>,
  keyboard_input: Res<Input<KeyCode>>,
  mut windows: ResMut<Windows>,
  mut egui_context: ResMut<EguiContext>,
  interned_textures: Res<InternedTextures>,
  mut spawn_model_events: ResMut<Events<SpawnModelEvent>>,
  model_query: Query<(Entity, &ModelInfo, &SceneDecomposition, &ModelParams)>,
  view_info: Res<ViewInfo>,
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
      for (model, model_info, decomp, model_params) in model_query.iter() {
        let texture_id = if let Some(texture_id) = interned_textures.get_egui_id(&model_info.name) {
          texture_id
        } else {
          interned_textures.null_texture()
        };

        let thumbnail = ui.add(egui::widgets::ImageButton::new(
          egui::TextureId::User(texture_id),
          [100.0, 100.0],
        ));

        if thumbnail.clicked {
          let aabb = decomp.aabb(&model_params.scale.to_na_vector3());
          let half_height = aabb.half_extents().y;
          let mut translation = view_info
            .hit_point()
            .unwrap_or_else(|| view_info.ray.point_at(half_height));
          translation += Vector3::new(0., half_height, 0.);
          let position = Isometry3::from_parts(
            Translation3::from(translation.coords),
            UnitQuaternion::identity(),
          );
          spawn_model_events.send(SpawnModelEvent { model, position });
        }
      }
    });
  }
}

fn debugger_system(world: &mut World, resources: &mut Resources) {
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
      WorldUIContext::new(world, resources).ui(ui, &Default::default());
    });
  }
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_plugin(EguiPlugin)
      .init_resource::<InspectableRegistry>()
      .init_resource::<InternedTextures>()
      .add_system(spawn_ui_system.system())
      .add_system(debugger_system.system())
      .add_system_to_stage(stage::POST_UPDATE, load_assets.system());
  }
}
