use crate::{physics::ColliderParams, prelude::*};
use bevy_rapier3d::{na::Isometry3, rapier::dynamics::BodyStatus};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod decomposition;
mod thumbnail;

pub use decomposition::{MeshDecomposition, SceneDecomposition};
pub use thumbnail::Thumbnail;

#[derive(Clone, Serialize, Deserialize)]
pub struct ModelParams {
  #[serde(default)]
  pub scale: Vec3,
  #[serde(default)]
  pub mass: f32,
}

impl Default for ModelParams {
  fn default() -> Self {
    ModelParams {
      scale: Vec3::new(1., 1., 1.),
      mass: 1.0,
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

#[derive(Debug)]
pub struct SpawnModelEvent {
  pub model: Entity,
  pub position: Isometry3<f32>,
}

fn listen_for_spawn_models(
  commands: &mut Commands,
  mut event_reader: EventReader<SpawnModelEvent>,
  query: Query<(&ModelInfo, &ModelParams, &Handle<Scene>)>,
) {
  for event in event_reader.iter() {
    let SpawnModelEvent { model, position } = &event;
    let (model_info, params, scene_handle) = query.get(*model).unwrap();
    commands
      .spawn((
        Transform::from_matrix(Mat4::from_scale_rotation_translation(
          params.scale,
          position.rotation.to_glam_quat(),
          position.translation.vector.to_glam_vec3(),
        )),
        GlobalTransform::default(),
        ColliderParams {
          body_status: BodyStatus::Dynamic,
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
      .add_event::<SpawnModelEvent>()
      .add_system(thumbnail::load_thumbnail.system())
      .add_system(decomposition::load_decomp.system())
      .add_system(listen_for_spawn_models.system());
  }
}
