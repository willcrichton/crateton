use crate::{json::JsonLoader, physics::ColliderParams, prelude::*};
use bevy_rapier3d::{na::Isometry3, rapier::dynamics::BodyStatus};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod decomposition;
mod thumbnail;

pub use decomposition::{MeshDecomposition, SceneDecomposition};
pub use thumbnail::Thumbnail;

fn scale_default() -> Vec3 {
  Vec3::new(1., 1., 1.)
}

fn mass_default() -> f32 {
  1.
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ModelParams {
  #[serde(default = "scale_default")]
  pub scale: Vec3,
  #[serde(default = "mass_default")]
  pub mass: f32,
}

impl Default for ModelParams {
  fn default() -> Self {
    ModelParams {
      scale: scale_default(),
      mass: mass_default(),
    }
  }
}

pub struct ModelInstance(pub Entity);

#[derive(Clone, Debug)]
pub struct ModelInfo {
  pub name: String,
  pub path: String,
}

impl ModelInfo {
  fn dir(&self) -> &Path {
    Path::new(&self.path).parent().unwrap()
  }

  pub fn thumbnail_path(&self) -> PathBuf {
    self.dir().join("thumbnail.jpg")
  }

  pub fn mesh_decomposition_path(&self) -> PathBuf {
    self.dir().join("mesh_decomposition.json")
  }

  pub fn params_path(&self) -> PathBuf {
    self.dir().join("config.json")
  }
}

struct ModelCategory(Entity);

fn model_init(commands: &mut Commands, mut category: ResMut<ModelCategory>) {
  let entity = commands
    .spawn((Name::new("Models"),))
    .current_entity()
    .unwrap();
  category.0 = entity;
}

pub struct LoadModelEvent {
  pub path: String,
}

fn listen_for_load_models(
  commands: &mut Commands,
  asset_server: Res<AssetServer>,
  mut json_loader: ResMut<JsonLoader>,
  mut event_reader: EventReader<LoadModelEvent>,
  category: Res<ModelCategory>,
) {
  let io = asset_server.io();
  for LoadModelEvent { path } in event_reader.iter() {
    let path = path.to_string();
    let name = Path::new(&path)
      .file_stem()
      .unwrap()
      .to_str()
      .unwrap()
      .to_string();
    let scene: Handle<Scene> = asset_server.load(path.as_str());
    let model_info = ModelInfo { name, path };
    commands.spawn((scene, model_info.clone(), Parent(category.0)));

    if io.exists(&model_info.params_path()) {
      json_loader.load::<ModelParams>(commands, asset_server.load(model_info.params_path()));
    } else {
      commands.with(ModelParams::default());
    }
  }
}

#[derive(Debug)]
pub struct SpawnModelEvent {
  pub model: Entity,
  pub position: Isometry3<f32>,
  pub body_status: BodyStatus,
}

fn listen_for_spawn_models(
  commands: &mut Commands,
  mut event_reader: EventReader<SpawnModelEvent>,
  query: Query<(&ModelInfo, &ModelParams, &Handle<Scene>)>,
) {
  for event in event_reader.iter() {
    let SpawnModelEvent {
      model,
      position,
      body_status,
    } = &event;
    let (model_info, params, scene_handle) = query.get(*model).unwrap();
    // info!("initial position {:#?}", position);
    // info!("inital scale: {:#?}", params.scale);
    // info!("position.rotation {:?}, to _glam quat {:?}", position.rotation, position.rotation.to_glam_quat());
    // info!("spawned with {:#?}", Transform::from_matrix(Mat4::from_scale_rotation_translation(
    //   params.scale,
    //   position.rotation.to_glam_quat(),
    //   position.translation.vector.to_glam_vec3(),
    // )));
    commands
      .spawn((
        Transform::from_matrix(Mat4::from_scale_rotation_translation(
          params.scale,
          position.rotation.to_glam_quat(),
          position.translation.vector.to_glam_vec3(),
        )),
        GlobalTransform::identity(),
        ColliderParams {
          body_status: *body_status,
          mass: params.mass,
        },
        ModelInstance(*model),
        Name::new(&model_info.name),
      ))
      .with_children(|parent| {
        parent.spawn_scene(scene_handle.clone());
      });
  }
}

pub struct ModelsPlugin;
impl Plugin for ModelsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_resource(ModelCategory(Entity::from_bits(0)))
      .add_event::<SpawnModelEvent>()
      .add_event::<LoadModelEvent>()
      .add_startup_system(model_init.system())
      .add_system(thumbnail::load_thumbnail.system())
      .add_system(decomposition::load_decomp.system())
      .add_system(listen_for_spawn_models.system())
      .add_system(listen_for_load_models.system());
  }
}
