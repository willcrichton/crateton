use super::ModelInfo;
use crate::prelude::*;
use std::process::Command;

pub struct Thumbnail(pub Handle<Texture>);

pub fn load_thumbnail(
  commands: &mut Commands,
  query: Query<(Entity, &ModelInfo), Without<Thumbnail>>,
  asset_server: Res<AssetServer>,
) {
  let io = asset_server.io();
  for (entity, model_info) in query.iter() {
    let thumbnail_path = model_info.thumbnail_path();
    info!("Loading thumbnail for {}", model_info.name);
    if !io.exists(&thumbnail_path) {
      Command::new("cargo")
        .args(&["run", "--bin", "generate_thumbnail", "--", &model_info.path])
        .status()
        .unwrap();
    }

    let thumbnail = asset_server.load(thumbnail_path);
    commands.insert_one(entity, Thumbnail(thumbnail));
  }
}
