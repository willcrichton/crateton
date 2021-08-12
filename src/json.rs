use crate::prelude::*;
use bevy::reflect as bevy_reflect;
use bevy::{
  asset::{AssetLoader, LoadContext, LoadedAsset},
  ecs::system::EntityCommands,
  reflect::TypeUuid,
  utils::BoxedFuture,
};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(TypeUuid)]
#[uuid = "e37c93d2-e55f-42ba-8ba4-ee063768b4f8"]
pub struct JsonData(serde_json::Value);

#[derive(Default)]
struct JsonAssetLoader;
impl AssetLoader for JsonAssetLoader {
  fn load<'a>(
    &'a self,
    bytes: &'a [u8],
    load_context: &'a mut LoadContext,
  ) -> BoxedFuture<'a, anyhow::Result<()>> {
    Box::pin(async move {
      let params: serde_json::Value = serde_json::from_slice(bytes)?;
      load_context.set_default_asset(LoadedAsset::new(JsonData(params)));
      Ok(())
    })
  }

  fn extensions(&self) -> &[&str] {
    &["json"]
  }
}

type JsonCallback = Box<dyn Fn(&mut Commands, Entity, &JsonData) -> () + Send + Sync>;

struct ValueLoader {
  handle: Handle<JsonData>,
  convert: JsonCallback,
}

impl ValueLoader {
  fn new<T: DeserializeOwned + Send + Sync + 'static>(handle: Handle<JsonData>) -> Self {
    let convert = Box::new(
      |commands: &mut Commands, entity: Entity, json_data: &JsonData| {
        let data: T = serde_json::from_value(json_data.0.clone()).unwrap();
        commands
          .entity(entity)
          .insert(data)
          .remove::<LoadingJsonTag<T>>();
      },
    ) as JsonCallback;
    ValueLoader { handle, convert }
  }
}
#[derive(Default)]
pub struct JsonLoader(HashMap<Entity, Vec<ValueLoader>>);
impl JsonLoader {
  pub fn load<T: DeserializeOwned + Send + Sync + 'static>(
    &mut self,
    mut commands: EntityCommands,
    handle: Handle<JsonData>,
  ) {
    commands.insert(LoadingJsonTag::<T>(PhantomData));
    let loaders = self.0.entry(commands.id()).or_insert_with(Vec::new);
    loaders.push(ValueLoader::new::<T>(handle));
  }
}

pub struct LoadingJsonTag<T>(PhantomData<T>);

fn load_json(
  mut commands: Commands,
  assets: Res<Assets<JsonData>>,
  mut json_loader: ResMut<JsonLoader>,
) {
  for (entity, loaders) in json_loader.0.iter_mut() {
    let mut to_delete = vec![];
    for (i, loader) in loaders.iter().enumerate() {
      if let Some(data) = assets.get(loader.handle.clone()) {
        (loader.convert)(&mut commands, *entity, data);
        to_delete.push(i);
      }
    }

    to_delete.reverse();
    for i in to_delete {
      loaders.remove(i);
    }
  }
}

pub struct JsonPlugin;
impl Plugin for JsonPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<JsonLoader>()
      .add_asset::<JsonData>()
      .init_asset_loader::<JsonAssetLoader>()
      .add_system(load_json.system());
  }
}
