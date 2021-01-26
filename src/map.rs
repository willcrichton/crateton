use crate::{
  assets::{AssetRegistry, AssetState, ASSET_STAGE},
  json::*,
  models::*,
  physics::{ColliderParams, MeshWrapper},
  player::raycast::ViewInfo,
  prelude::*,
};
use bevy::{
  asset::{AssetLoader, LoadContext, LoadedAsset},
  reflect::TypeUuid,
  tasks::TaskPoolBuilder,
};

use std::marker::PhantomData;

use bevy_rapier3d::rapier::dynamics::BodyStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

fn init_map(
  commands: &mut Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
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
      body_status: BodyStatus::Static,
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
      body_status: BodyStatus::Dynamic,
      mass: 1.0,
    },
    Name::new("box"),
  ));

  // let monkey = &map_assets.models["Duck"];
  // monkey.spawn(
  //   commands,
  //   &json_assets,
  //   Isometry3::from_parts(
  //     Translation3::from(Vector3::new(0., 100., 0.)),
  //     UnitQuaternion::identity(),
  //   ),
  // );
  // commands.with(Name::new("duck"));
}

fn load_map_assets(
  commands: &mut Commands,
  //mut map_assets: ResMut<MapAssets>,
  //mut asset_registry: ResMut<AssetRegistry>,
  asset_server: Res<AssetServer>,
  mut json_loader: ResMut<JsonLoader>,
) {
  let model_paths = vec![
    "models/monkey/Monkey.gltf#Scene0",
    //"models/car/car.gltf#Scene0",
    "models/FlightHelmet/FlightHelmet.gltf#Scene0",
    "models/Duck/Duck.gltf#Scene0",
  ];

  let model_parent = commands
    .spawn((Name::new("Models"),))
    .with_children(|parent| {
      let io = asset_server.io();
      for path in model_paths.into_iter() {
        let path = path.to_string();
        let name = Path::new(&path)
          .file_stem()
          .unwrap()
          .to_str()
          .unwrap()
          .to_string();
        let scene: Handle<Scene> = asset_server.load(path.as_str());
        let model_info = ModelInfo { name, path };
        parent.spawn((scene, model_info.clone()));

        if io.exists(&model_info.params_path()) {
          json_loader
            .load_child::<ModelParams>(parent, asset_server.load(model_info.params_path()));
        } else {
          parent.with(ModelParams::default());
        }
      }
    });
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_startup_system(load_map_assets.system())
      .on_state_enter(ASSET_STAGE, AssetState::Finished, init_map.system());
  }
}
