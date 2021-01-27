use super::ModelInfo;
use crate::json::*;
use crate::{physics::MeshWrapper, prelude::*};
use bevy_rapier3d::na::{Point3, Vector3};
use ncollide3d::{
  bounding_volume::AABB,
  procedural::{IndexBuffer, TriMesh as NTriMesh},
  shape::TriMesh as NSTriMesh,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct MeshDecomposition(Vec<(Vec<Vec3>, Vec<UVec3>)>);

impl MeshDecomposition {
  pub fn to_trimesh(&self, scale: &Vector3<f32>) -> Vec<NTriMesh<f32>> {
    self
      .0
      .iter()
      .map(|(coords, indices)| {
        NTriMesh::new(
          coords
            .iter()
            .map(|coord| Point3::from(coord.to_na_point3().coords.component_mul(&scale)))
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
  pub fn aabb(&self, scale: &Vector3<f32>) -> AABB<f32> {
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

    AABB::new(
      Point3::from(mins.coords.component_mul(scale)),
      Point3::from(maxs.coords.component_mul(scale)),
    )
  }
}

pub fn load_decomp(
  commands: &mut Commands,
  mut query: Query<
    (Entity, &ModelInfo, &Handle<Scene>),
    Without<LoadingJsonTag<SceneDecomposition>>,
  >,
  scenes: Res<Assets<Scene>>,
  asset_server: Res<AssetServer>,
  meshes: Res<Assets<Mesh>>,
  mut json_loader: ResMut<JsonLoader>,
) {
  let io = asset_server.io();
  for (entity, model_info, scene_handle) in query.iter_mut() {
    let scene = if let Some(scene) = scenes.get(scene_handle.clone()) {
      scene
    } else {
      continue;
    };

    let mesh_decomposition_path = model_info.mesh_decomposition_path();
    if !io.exists(&mesh_decomposition_path) {
      let mut decomp = SceneDecomposition::default();
      for (child, mesh_handle) in scene.world.query::<(Entity, &Handle<Mesh>)>() {
        let mesh = meshes.get(mesh_handle.clone()).unwrap();
        let child_decomp = MeshWrapper::new(
          mesh,
          "Vertex_Position",
          "Vertex_Normal",
          Vector3::new(1., 1., 1.),
        )
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
        .collect();
        decomp.meshes.insert(child, MeshDecomposition(child_decomp));
      }

      let mut output_file =
        std::fs::File::create(&Path::new("assets").join(&mesh_decomposition_path)).unwrap();
      serde_json::to_writer(&mut output_file, &decomp).unwrap();
    }

    commands.set_current_entity(entity);
    json_loader.load::<SceneDecomposition>(commands, asset_server.load(mesh_decomposition_path));
  }
}
