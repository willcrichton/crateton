use anyhow::{Context, Result};
use bevy::{app::AppExit, gltf::GltfId, prelude::*};
use bevy_rapier3d::prelude::*;
use crateton::{
  models::mesh_wrapper::MeshWrapper,
  physics::{SceneDecomposition, NORMAL_ATTRIBUTE, POSITION_ATTRIBUTE},
};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn path() -> PathBuf {
  PathBuf::from(env::args().skip(1).next().unwrap())
}

fn main() {
  App::new()
    .insert_resource(Msaa { samples: 4 })
    .add_plugins(DefaultPlugins)
    // .add_plugins(MinimalPlugins)
    // .add_plugin(bevy::asset::AssetPlugin::default())
    // .add_plugin(bevy::gltf::GltfPlugin::default())
    // .add_plugin(bevy::transform::TransformPlugin::default())
    // .add_plugin(bevy::scene::ScenePlugin::default())
    .add_startup_system(setup.system())
    .add_system(init_physics.system())
    // .add_system_to_stage(CoreStage::Last, wait_for_spawn.system())
    .run();
}

fn init_physics(
  query: Query<(&GltfId, &Handle<Mesh>)>,
  meshes: Res<Assets<Mesh>>,
  mut exit: EventWriter<AppExit>,
) {
  if query.is_empty() {
    return;
  }

  let mut decomp_map = HashMap::new();
  for (id, handle) in query.iter() {
    let mesh = meshes.get(handle).unwrap();
    let wrapper = MeshWrapper::new(mesh, POSITION_ATTRIBUTE, NORMAL_ATTRIBUTE);
    let decomp = ColliderShape::convex_decomposition(&wrapper.vertices(), &wrapper.indices());
    let meshes = decomp.as_compound().unwrap().shapes().iter().map(|(offset, poly)| {
      let (vertices, indices) = poly.as_convex_polyhedron().unwrap().to_trimesh();
      (offset.clone(), vertices, indices)
    }).collect::<Vec<_>>();
    decomp_map.insert(*id, meshes);
  }

  let scene_decomp = SceneDecomposition(decomp_map);
  let inner = || -> Result<()> {
    let bytes = rmp_serde::to_vec(&scene_decomp)?;
    let out_path = PathBuf::from("assets")
      .join(path().parent().context("No parent")?)
      .join("mesh_decomposition.rmp");
    let mut f = File::create(out_path)?;
    f.write_all(&bytes)?;
    Ok(())
  };

  inner().unwrap();
  exit.send(AppExit);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands
    .spawn_bundle((GlobalTransform::identity(), Transform::identity()))
    .with_children(|parent| {
      parent.spawn_scene(asset_server.load(format!("{}#Scene0", path().display()).as_str()));
    });
}
