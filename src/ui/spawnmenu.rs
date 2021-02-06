use crate::{
  models::{ModelInfo, SceneDecomposition, SpawnModelEvent, Thumbnail},
  player::{controller::CharacterController, raycast::ViewInfo},
  prelude::*,
};

use bevy_egui::{egui, EguiContext};
use bevy_rapier3d::{
  na::{Isometry3, Translation3, UnitQuaternion, Vector3},
  rapier::dynamics::BodyStatus,
};

use super::{InternedTextures, UiLock, UiWindowManager};

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
  mut egui_context: ResMut<EguiContext>,
  interned_textures: Res<InternedTextures>,
  mut spawn_model_events: ResMut<Events<SpawnModelEvent>>,
  model_query: Query<(Entity, &ModelInfo, &SceneDecomposition)>,
  view_info: Res<ViewInfo>,
  mut ui_window_manager: ResMut<UiWindowManager>,
  mut ui_lock: Local<Option<UiLock>>
) {
  let key = controller.input_map.key_show_ui;
  if keyboard_input.just_pressed(key) {
    *ui_lock = ui_window_manager.try_show();
  } else if keyboard_input.just_released(key) && ui_lock.is_some() {
    let lock = ui_lock.take().unwrap();
    ui_window_manager.unshow(lock);
  }

  if keyboard_input.pressed(key) {
    let ctx = &mut egui_context.ctx;
    egui::Window::new("Spawn window").show(ctx, |ui| {
      for (model, model_info, decomp) in model_query.iter() {
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
          let aabb = decomp.aabb();
          let half_height = aabb.half_extents().y;
          let mut translation = view_info
            .hit_point()
            .unwrap_or_else(|| view_info.ray.point_at(half_height));
          translation += Vector3::new(0., half_height, 0.);
          let position = Isometry3::from_parts(
            Translation3::from(translation.coords),
            UnitQuaternion::identity(),
          );
          spawn_model_events.send(SpawnModelEvent {
            model,
            position,
            body_status: BodyStatus::Dynamic,
          });
        }
      }
    });
  }
}

pub struct SpawnmenuPlugin;
impl Plugin for SpawnmenuPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_system(spawn_ui_system.system())
      .add_system_to_stage(stage::POST_UPDATE, load_assets.system());
  }
}
