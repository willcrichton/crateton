use super::{ModelInfo, ModelParams};
use crate::json::*;
use crate::{physics::MeshWrapper, prelude::*};
use bevy::transform::transform_propagate_system::transform_propagate_system;
use bevy_rapier3d::na::{point, Point3};
use bevy_rapier3d::rapier::parry::{bounding_volume::AABB, shape::TriMesh};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct MeshDecomposition(Vec<(Vec<Vec3>, Vec<UVec3>)>);

impl MeshDecomposition {
  pub fn to_trimesh(&self) -> Vec<TriMesh> {
    self
      .0
      .iter()
      .map(|(coords, indices)| {
        TriMesh::new(
          coords
            .iter()
            .map(|coord| Point3::from(coord.to_na_point3()))
            .collect(),
          indices
            .iter()
            .map(|idxs| idxs.to_na_point3().coords.into())
            .collect(),
        )
      })
      .collect()
  }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SceneDecomposition {
  pub meshes: HashMap<Entity, MeshDecomposition>,
}

impl SceneDecomposition {
  pub fn aabb(&self) -> AABB {
    let mut mins = point![f32::MAX, f32::MAX, f32::MAX];
    let mut maxs = point![f32::MIN, f32::MIN, f32::MIN];
    for mesh in self.meshes.values() {
      for (coords, _) in &mesh.0 {
        for coord in coords {
          mins = mins.inf(&coord.to_na_point3());
          maxs = maxs.sup(&coord.to_na_point3());
        }
      }
    }

    AABB::new(mins, maxs)
  }
}

fn update_global_transform(world: &mut World) {
  let mut update_stage = SystemStage::serial();
  update_stage.add_system(transform_propagate_system.system());
  let mut schedule = Schedule::default();
  schedule.add_stage("update", update_stage);
  schedule.run_once(world);
}

fn compute_mesh_decomposition(mesh: MeshWrapper) -> MeshDecomposition {
  MeshDecomposition(
    mesh
      .compute_decomposition(None)
      .into_iter()
      .map(|trimesh| {
        let indices = trimesh
          .indices()
          .iter()
          .map(|&[x, y, z]| UVec3::new(x, y, z))
          .collect();
        let coords = trimesh
          .vertices()
          .iter()
          .map(|point| point.to_glam_vec3())
          .collect();
        (coords, indices)
      })
      .collect(),
  )
}

fn compute_scene_decomposition(
  world: &mut World,
  model_params: &ModelParams,
  meshes: &Assets<Mesh>,
) -> SceneDecomposition {
  let mut decomp = SceneDecomposition::default();
  for (child, mesh_handle, transform) in world.query::<(Entity, &Handle<Mesh>, &GlobalTransform)>()
  {
    let mesh = meshes.get(mesh_handle.clone()).unwrap();
    let scale = transform.scale * model_params.scale;
    let offset = transform.translation;
    decomp.meshes.insert(
      child,
      compute_mesh_decomposition(MeshWrapper::new(
        mesh,
        "Vertex_Position",
        "Vertex_Normal",
        scale.to_na_vector3(),
        offset.to_na_vector3(),
      )),
    );
  }
  decomp
}

pub fn load_decomp(
  mut commands: Commands,
  mut query: Query<
    (Entity, &ModelInfo, &ModelParams, &Handle<Scene>),
    Without<LoadingJsonTag<SceneDecomposition>>,
  >,
  mut scenes: ResMut<Assets<Scene>>,
  asset_server: Res<AssetServer>,
  meshes: Res<Assets<Mesh>>,
  mut json_loader: ResMut<JsonLoader>,
) {
  let io = asset_server.io();
  for (entity, model_info, model_params, scene_handle) in query.iter_mut() {
    let scene = if let Some(scene) = scenes.get_mut(scene_handle.clone()) {
      scene
    } else {
      continue;
    };

    update_global_transform(&mut scene.world);

    let mesh_decomposition_path = model_info.mesh_decomposition_path();
    if !io.exists(&mesh_decomposition_path) {
      info!("Computing decomposition for: {}", model_info.name);
      let scene_decomposition =
        compute_scene_decomposition(&mut scene.world, model_params, &meshes);
      let mut output_file =
        std::fs::File::create(&Path::new("assets").join(&mesh_decomposition_path)).unwrap();
      serde_json::to_writer(&mut output_file, &scene_decomposition).unwrap();
    }

    commands.set_current_entity(entity);
    json_loader.load::<SceneDecomposition>(commands, asset_server.load(mesh_decomposition_path));
  }
}
