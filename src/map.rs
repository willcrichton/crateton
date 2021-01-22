use bevy::{
  asset::{AssetLoader, LoadContext, LoadedAsset},
  reflect::TypeUuid,
  utils::BoxedFuture,
};
use bevy_rapier3d::na::{Isometry3, Point3, Translation3, UnitQuaternion, Vector3};
use ncollide3d::bounding_volume::AABB;

use crate::{
  assets::{AssetRegistry, AssetState, ASSET_STAGE},
  physics::{AltBodyStatus, ColliderParams, MeshWrapper},
  player::raycast::ViewInfo,
  prelude::*,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ModelParams {
  pub scale: Option<Vec3>,
  pub mass: Option<f32>,
}

#[derive(TypeUuid)]
#[uuid = "e37c93d2-e55f-42ba-8ba4-ee063768b4f8"]
pub struct JsonData(pub serde_json::Value);

#[derive(Default)]
pub struct JsonLoader;
impl AssetLoader for JsonLoader {
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

#[derive(Debug, Clone, Default)]
pub struct Model {
  scene: Handle<Scene>,
  params: Handle<JsonData>,
}

impl Model {
  pub fn scene(&self) -> &Handle<Scene> {
    &self.scene
  }

  pub fn params(&self, json_assets: &Assets<JsonData>) -> ModelParams {
    json_assets
      .get(self.params.clone())
      .map(|data| serde_json::from_value(data.0.clone()).unwrap())
      .unwrap_or_else(|| ModelParams::default())
  }

  pub fn aabb(&self, scenes: &Assets<Scene>, meshes: &Assets<Mesh>, json_assets: &Assets<JsonData>) -> AABB<f32> {
    let params = self.params(json_assets);
    let scene = scenes.get(self.scene().clone()).unwrap();
    scene
      .world
      .query::<Entity>()
      .filter_map(|entity| {
        scene
          .world
          .get::<Handle<Mesh>>(entity)
          .map(|mesh_handle| {
            let mesh = meshes.get(mesh_handle).unwrap();
            MeshWrapper::new(
              mesh,
              "Vertex_Position",
              "Vertex_Normal",
              params.scale.unwrap_or_else(|| Vec3::new(1., 1., 1.)).to_na_vector3()
            )
            .aabb()
          })
          .ok()
      })
      .fold(
        AABB::new(Point3::origin(), Point3::origin()),
        |aabb1, aabb2| AABB::new(aabb1.mins.inf(&aabb2.mins), aabb1.maxs.sup(&aabb2.maxs)),
      )
  }

  pub fn spawn(
    &self,
    commands: &mut Commands,
    json_assets: &Assets<JsonData>,
    position: Isometry3<f32>,
  ) {
    let params = self.params(json_assets);
    commands
      .spawn((
        Transform::from_matrix(Mat4::from_scale_rotation_translation(
          params.scale.unwrap_or_else(|| Vec3::new(1., 1., 1.)),
          position.rotation.to_glam_quat(),
          position.translation.vector.to_glam_vec3(),
        )),
        GlobalTransform::default(),
        ColliderParams {
          body_status: AltBodyStatus::Dynamic,
          mass: params.mass.unwrap_or(1.0),
        },
      ))
      .with_children(|parent| {
        parent.spawn_scene(self.scene.clone());
      });
  }
}

#[derive(Debug, Default)]
pub struct MapAssets {
  pub models: HashMap<String, Model>,
  pub thumbnails: HashMap<String, Handle<Texture>>,
}

fn init_map(
  commands: &mut Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  map_assets: Res<MapAssets>,
  json_assets: Res<Assets<JsonData>>,
) {
  let lights = vec![LightBundle {
    transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
    ..Default::default()
  }];
  for (i, light) in lights.into_iter().enumerate() {
    commands
      .spawn(light)
      .with(Name::new(format!("light {}", i)));
  }

  let ground_size = 200.1;
  let ground_height = 1.0;
  let extents = Vec3::new(0.5 * ground_size, 0.5 * ground_height, 0.5 * ground_size);
  let cube = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));
  let color = Color::rgb(
    0xF3 as f32 / 255.0,
    0xD9 as f32 / 255.0,
    0xB1 as f32 / 255.0,
  );
  let ground = PbrBundle {
    mesh: cube.clone(),
    transform: Transform::from_scale(extents),
    material: materials.add(color.into()),
    ..Default::default()
  };
  commands.spawn(ground);
  commands.with_bundle((
    ColliderParams {
      body_status: AltBodyStatus::Static,
      mass: 10000.0,
    },
    Name::new("ground"),
  ));

  let box_ = PbrBundle {
    mesh: cube.clone(),
    material: materials.add(Color::rgb(1., 0., 0.3).into()),
    transform: Transform::from_translation(Vec3::new(0., 7., 0.)),
    // render_pipelines: RenderPipelines::from_pipelines(vec![
    //   //RenderPipeline::new(FORWARD_PIPELINE_HANDLE.typed()),
    //   RenderPipeline::new(pipeline_handle),
    // ]),
    ..Default::default()
  };
  commands.spawn(box_);
  commands.with_bundle((
    ColliderParams {
      body_status: AltBodyStatus::Dynamic,
      mass: 1.0,
    },
    Name::new("box"),
  ));

  let monkey = &map_assets.models["Duck"];
  monkey.spawn(
    commands,
    &json_assets,
    Isometry3::from_parts(
      Translation3::from(Vector3::new(0., 100., 0.)),
      UnitQuaternion::identity(),
    ),
  );
  commands.with(Name::new("duck"));
}

// if asset_registry
//   .scene_instances
//   .iter()
//   .all(|inst| scene_spawner.instance_is_ready(*inst))
// {
//   state.set_next(AssetState::Finished).unwrap();
// }

fn load_map_assets(
  mut map_assets: ResMut<MapAssets>,
  mut asset_registry: ResMut<AssetRegistry>,
  asset_server: Res<AssetServer>,
) {
  let (models, thumbnails) = vec![
    "models/monkey/Monkey.gltf#Scene0",
    //"models/car/car.gltf#Scene0",
    "models/FlightHelmet/FlightHelmet.gltf#Scene0",
    "models/Duck/Duck.gltf#Scene0",
  ]
  .into_iter()
  .map(|path| {
    let scene = asset_registry.register_model(&asset_server, &path);
    let path = Path::new(path);
    let parent = path.parent().unwrap();
    let thumbnail_path = parent.join("thumbnail.jpg");
    let thumbnail =
      asset_registry.register_texture(&asset_server, thumbnail_path.to_str().unwrap());
    let config_path = parent.join("config.json");
    let params = asset_server.load(config_path);
    let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
    ((stem.clone(), Model { scene, params }), (stem, thumbnail))
  })
  .unzip();
  map_assets.models = models;
  map_assets.thumbnails = thumbnails;
}

#[derive(Debug)]
pub struct SpawnModelEvent {
  pub model_name: String,
}

fn listen_for_spawn_models(
  commands: &mut Commands,
  mut event_reader: EventReader<SpawnModelEvent>,
  map_assets: Res<MapAssets>,
  view_info: Res<ViewInfo>,
  meshes: Res<Assets<Mesh>>,
  scenes: Res<Assets<Scene>>,
  json_assets: Res<Assets<JsonData>>,
) {
  for event in event_reader.iter() {
    let model = &map_assets.models[&event.model_name];

    let aabb = model.aabb(&scenes, &meshes, &json_assets);
    let half_height = aabb.half_extents().y;
    println!("{:?}, {:?}", aabb, half_height);

    let mut translation = view_info
      .hit_point()
      .unwrap_or_else(|| view_info.ray.point_at(half_height));
    translation += Vector3::new(0., half_height + 2., 0.);

    model.spawn(
      commands,
      &json_assets,
      Isometry3::from_parts(
        Translation3::from(translation.coords),
        UnitQuaternion::from_euler_angles(0., 0., 0.)
        //UnitQuaternion::identity(),
      ),
    );
    commands.with(Name::new(&event.model_name));
  }
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_asset::<JsonData>()
      .init_asset_loader::<JsonLoader>()
      .add_event::<SpawnModelEvent>()
      .register_type::<ColliderParams>()
      .init_resource::<MapAssets>()
      .add_startup_system(load_map_assets.system())
      .add_system(listen_for_spawn_models.system())
      .on_state_enter(ASSET_STAGE, AssetState::Finished, init_map.system());
  }
}
