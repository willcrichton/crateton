use crate::{
  assets::{AssetRegistry, AssetState, ASSET_STAGE},
  physics::{AltBodyStatus, ColliderParams},
};

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;

#[derive(Default)]
pub struct MapAssets {
  pub models: HashMap<String, Handle<Scene>>,
  pub thumbnails: HashMap<String, Handle<Texture>>,
}

fn init_map(
  commands: &mut Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  map_assets: Res<MapAssets>,
  mut scenes: ResMut<Assets<Scene>>,
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

  for model in map_assets.models.values() {
    let scene = scenes.get_mut(model.clone()).unwrap();
    let mut scene_commands = Commands::default();
    for (entity, _) in scene.world.query::<(Entity, &Handle<Mesh>)>() {
      scene_commands.insert(
        entity,
        (
          // ColliderParams {
          //   body_status: AltBodyStatus::Dynamic,
          //   mass: 1.0,
          // },
          //Transform::identity(),
          GlobalTransform::identity(),
        ),
      );
    }
    scene_commands.apply(&mut scene.world, &mut Resources::default());
  }

  let monkey = map_assets.models["Monkey"].clone();
  commands
    .spawn((
      Name::new("monkey"),
      Transform::from_translation(Vec3::new(0., 10., 0.)),
      GlobalTransform::default(),
      ColliderParams {
        body_status: AltBodyStatus::Dynamic,
        mass: 1.0,
      },
    ))
    .with_children(|parent| {
      parent.spawn_scene(monkey);
    });
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
  ]
  .into_iter()
  .map(|path| {
    let model = asset_registry.register_model(&asset_server, &path);
    let path = Path::new(path);
    let thumbnail_path = path.parent().unwrap().join("thumbnail.jpg");
    let thumbnail =
      asset_registry.register_texture(&asset_server, thumbnail_path.to_str().unwrap());
    let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
    ((stem.clone(), model), (stem, thumbnail))
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
) {
  for event in event_reader.iter() {
    let model = map_assets.models[&event.model_name].clone();
    commands.spawn_scene(model);
  }
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_event::<SpawnModelEvent>()
      .register_type::<ColliderParams>()
      .init_resource::<MapAssets>()
      .add_startup_system(load_map_assets.system())
      .add_system(listen_for_spawn_models.system())
      .on_state_enter(ASSET_STAGE, AssetState::Finished, init_map.system());
  }
}
