use super::{ModelInfo, ModelParams, SceneDecomposition};
use crate::prelude::*;
use std::process::Command;

pub struct Thumbnail(pub Handle<Texture>);

pub fn load_thumbnail(
  commands: &mut Commands,
  query: Query<(Entity, &ModelInfo, &ModelParams, &SceneDecomposition), Without<Thumbnail>>,
  asset_server: Res<AssetServer>,
) {
  let io = asset_server.io();
  for (entity, model_info, model_params, decomposition) in query.iter() {
    let aabb = decomposition.aabb();
    let center = aabb.center();
    let half_extents = aabb.half_extents();
    let scale = &model_params.scale;

    let thumbnail_path = model_info.thumbnail_path();
    if !io.exists(&thumbnail_path) {
      Command::new("cargo")
        .args(&[
          "run",
          "--bin",
          "generate_thumbnail",
          "--",
          &model_info.path,
          &center.x.to_string(),
          &center.y.to_string(),
          &center.z.to_string(),
          &half_extents.x.to_string(),
          &half_extents.y.to_string(),
          &half_extents.z.to_string(),
          &scale.x.to_string(),
          &scale.y.to_string(),
          &scale.z.to_string(),
        ])
        .status()
        .unwrap();
    }

    let thumbnail = asset_server.load(thumbnail_path);
    commands.insert_one(entity, Thumbnail(thumbnail));
  }
}
