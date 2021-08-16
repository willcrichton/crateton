use crate::{models::*, physics::ColliderParams, prelude::*};
use bevy_rapier3d::{
  na::{Isometry3, Translation3, UnitQuaternion, Vector3},
  rapier::dynamics::BodyStatus,
};

fn init_map(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut spawn_model_events: ResMut<Events<SpawnModelEvent>>,
  model_query: Query<(Entity, &ModelInfo), With<ModelParams>>,
  mut done: Local<bool>,
) {
  if *done {
    return;
  }

  let model = match model_query
    .iter()
    .find(|(_, info)| info.name == "WebsiteTerrain")
  {
    Some((model, _)) => model,
    None => {
      return;
    }
  };

  info!("Initializing map");
  *done = true;

  let lights = vec![PointLightBundle {
    transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
    ..Default::default()
  }];
  for (i, light) in lights.into_iter().enumerate() {
    commands
      .spawn_bundle(light)
      .insert(Name::new(format!("light {}", i)));
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
  commands.spawn_bundle(ground).insert_bundle((
    ColliderParams {
      body_status: BodyStatus::Static,
      mass: 10000.0,
    },
    Name::new("ground"),
  ));

  let position = Isometry3::from_parts(
    Translation3::from(Vector3::new(0., 0., 0.)),
    UnitQuaternion::identity(),
  );
  spawn_model_events.send(SpawnModelEvent {
    model,
    position,
    body_status: BodyStatus::Static,
  });
}

fn load_map_assets(mut events: ResMut<Events<LoadModelEvent>>) {
  let model_paths = vec![
    "models/monkey/Monkey.gltf#Scene0",
    //"models/car/car.gltf#Scene0",
    "models/FlightHelmet/FlightHelmet.gltf#Scene0",
    "models/Duck/Duck.gltf#Scene0",
    "models/website-terrain/WebsiteTerrain.gltf#Scene0",
  ];

  for path in model_paths {
    let path = path.to_string();
    events.send(LoadModelEvent { path });
  }
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_startup_system(load_map_assets.system())
      .add_system(init_map.system());
  }
}
