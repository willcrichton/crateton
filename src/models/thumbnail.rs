use bevy_rapier3d::na::Vector3;

use super::{ModelInfo, SceneDecomposition};
use crate::prelude::*;
use std::process::Command;

pub struct Thumbnail(pub Handle<Texture>);

pub fn load_thumbnail(
  commands: &mut Commands,
  query: Query<(Entity, &ModelInfo, &SceneDecomposition), Without<Thumbnail>>,
  asset_server: Res<AssetServer>,
) {
  let io = asset_server.io();
  for (entity, model_info, decomposition) in query.iter() {
    let aabb = decomposition.aabb(&Vector3::new(1., 1., 1.));
    let center = aabb.center();
    let half_extents = aabb.half_extents();
    info!("{:?} {:?} {:?}", center, half_extents, aabb);

    let thumbnail_path = model_info.thumbnail_path();
    info!("Loading thumbnail for {}", model_info.name);
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
        ])
        .status()
        .unwrap();
    }

    let thumbnail = asset_server.load(thumbnail_path);
    commands.insert_one(entity, Thumbnail(thumbnail));
  }
}
