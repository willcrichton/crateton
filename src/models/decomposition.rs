use super::{ModelInfo, ModelParams};
use crate::json::*;
use crate::{physics::MeshWrapper, prelude::*};
use bevy::transform::transform_propagate_system::transform_propagate_system;
use bevy_rapier3d::na::Point3;
use ncollide3d::{
  bounding_volume::AABB,
  procedural::{IndexBuffer, TriMesh as NTriMesh},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct MeshDecomposition(Vec<(Vec<Vec3>, Vec<UVec3>)>);

impl MeshDecomposition {
  pub fn to_trimesh(&self) -> Vec<NTriMesh<f32>> {
    self
      .0
      .iter()
      .map(|(coords, indices)| {
        NTriMesh::new(
          coords
            .iter()
            .map(|coord| Point3::from(coord.to_na_point3()))
            .collect(),
          None,
          None,
          Some(IndexBuffer::Unified(
            indices.iter().map(|idxs| idxs.to_na_point3()).collect(),
          )),
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
  pub fn aabb(&self) -> AABB<f32> {
    let mut mins = Point3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut maxs = Point3::new(f32::MIN, f32::MIN, f32::MIN);
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
  let mut resources = Resources::default();
  let mut update_stage = SystemStage::serial();
  update_stage.add_system(transform_propagate_system.system());
  let mut schedule = Schedule::default();
  schedule.add_stage("update", update_stage);
  schedule.initialize_and_run(world, &mut resources);
}

fn compute_mesh_decomposition(mesh: MeshWrapper) -> MeshDecomposition {
  MeshDecomposition(
    mesh
      .compute_decomposition(None)
      .into_iter()
      .map(|trimesh| {
        let indices = if let IndexBuffer::Unified(indices) = trimesh.indices {
          indices
            .into_iter()
            .map(|point| UVec3::new(point.x, point.y, point.z))
            .collect()
        } else {
          unimplemented!();
        };

        let coords = trimesh
          .coords
          .into_iter()
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
  commands: &mut Commands,
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
