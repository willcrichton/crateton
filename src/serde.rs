use crate::prelude::*;
use bevy::reflect as bevy_reflect;
use bevy::{
  asset::{AssetLoader, LoadContext, LoadedAsset},
  ecs::system::EntityCommands,
  reflect::TypeUuid,
  utils::BoxedFuture,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::marker::PhantomData;

pub trait SerdeFormat: Send + Sync + Default + 'static {
  fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T>;
  fn extensions() -> &'static [&'static str];
}

#[derive(Default)]
pub struct JsonFormat;
impl SerdeFormat for JsonFormat {
  fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T> {
    Ok(serde_json::from_slice(bytes)?)
  }

  fn extensions() -> &'static [&'static str] {
    &["json"]
  }
}

#[derive(Default)]
pub struct RmpFormat;
impl SerdeFormat for RmpFormat {
  fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T> {
    Ok(rmp_serde::from_read(bytes)?)
  }

  fn extensions() -> &'static [&'static str] {
    &["rmp"]
  }
}

#[derive(TypeUuid)]
#[uuid = "e37c93d2-e55f-42ba-8ba4-ee063768b4f8"]
pub struct RawData(Vec<u8>);

#[derive(Default)]
struct SerdeAssetLoader<F>(PhantomData<F>);
impl<F: SerdeFormat> AssetLoader for SerdeAssetLoader<F> {
  fn load<'a>(
    &'a self,
    bytes: &'a [u8],
    load_context: &'a mut LoadContext,
  ) -> BoxedFuture<'a, anyhow::Result<()>> {
    Box::pin(async move {
      load_context.set_default_asset(LoadedAsset::new(RawData(bytes.to_vec())));
      Ok(())
    })
  }

  fn extensions(&self) -> &[&str] {
    F::extensions()
  }
}

type Deserializer = Box<dyn Fn(&mut Commands, Entity, &RawData) -> () + Send + Sync>;

struct SingleDataLoader<F> {
  handle: Handle<RawData>,
  convert: Deserializer,
  _format: PhantomData<F>,
}

impl<F: SerdeFormat> SingleDataLoader<F> {
  fn new<T: DeserializeOwned + Send + Sync + 'static>(handle: Handle<RawData>) -> Self {
    let convert = Box::new(
      |commands: &mut Commands, entity: Entity, serialized_data: &RawData| {
        let data: T = F::deserialize(&serialized_data.0).unwrap();
        commands
          .entity(entity)
          .insert(data)
          .remove::<LoadingSerializedDataTag<T>>();
      },
    ) as Deserializer;
    SingleDataLoader {
      handle,
      convert,
      _format: PhantomData,
    }
  }
}
#[derive(Default)]
pub struct SerdeLoader<F>(HashMap<Entity, Vec<SingleDataLoader<F>>>);
impl<F: SerdeFormat> SerdeLoader<F> {
  pub fn load<T: DeserializeOwned + Send + Sync + 'static>(
    &mut self,
    commands: &mut EntityCommands,
    handle: Handle<RawData>,
  ) {
    commands.insert(LoadingSerializedDataTag::<T>(PhantomData));
    let loaders = self.0.entry(commands.id()).or_insert_with(Vec::new);
    loaders.push(SingleDataLoader::<F>::new::<T>(handle));
  }
}

pub type JsonLoader = SerdeLoader<JsonFormat>;
pub type RmpLoader = SerdeLoader<RmpFormat>;

pub struct LoadingSerializedDataTag<T>(PhantomData<T>);

fn load_data<F: SerdeFormat>(
  mut commands: Commands,
  assets: Res<Assets<RawData>>,
  mut loader: ResMut<SerdeLoader<F>>,
) {
  for (entity, loaders) in loader.0.iter_mut() {
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

fn register<F: SerdeFormat>(app: &mut App) {
  app
    .init_resource::<SerdeLoader<F>>()
    .init_asset_loader::<SerdeAssetLoader<F>>()
    .add_system(load_data::<F>.system());
}

pub struct SerdePlugin;
impl Plugin for SerdePlugin {
  fn build(&self, app: &mut App) {
    app.add_asset::<RawData>();
    register::<RmpFormat>(app);
    register::<JsonFormat>(app);
  }
}
