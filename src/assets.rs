use bevy::{asset::LoadState, prelude::*, scene::InstanceId};
use bevy_rapier3d::{
  na::Isometry3,
  rapier::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder},
};

use crate::physics::MeshWrapper;

#[derive(Clone, Debug)]
enum AssetState {
  Start,
  Loading,
  Spawning,
  Finished,
}
#[derive(Default)]
struct AssetHandles {
  handles: Vec<Handle<Scene>>,
  instances: Vec<InstanceId>,
}

fn load_assets(
  mut state: ResMut<State<AssetState>>,
  mut asset_handles: ResMut<AssetHandles>,
  asset_server: Res<AssetServer>,
  mut scene_spawner: ResMut<SceneSpawner>,
  scenes: ResMut<Assets<Scene>>,
) {
  match state.current() {
    AssetState::Start => {
      asset_handles.handles = vec![asset_server.load("models/Monkey.gltf#Scene0")];
      state.set_next(AssetState::Loading).unwrap();
    }
    AssetState::Loading => {
      if let LoadState::Loaded =
        asset_server.get_group_load_state(asset_handles.handles.iter().map(|h| h.id))
      {
        asset_handles.instances = asset_handles
          .handles
          .iter()
          .map(|handle| {
            debug_assert!(scenes.get(handle).is_some(), "scene isn't properly loaded");
            scene_spawner.spawn(handle.clone())
          })
          .collect();
        state.set_next(AssetState::Spawning).unwrap();
      }
    }
    AssetState::Spawning => {
      if asset_handles
        .instances
        .iter()
        .all(|inst| scene_spawner.instance_is_ready(*inst))
      {
        state.set_next(AssetState::Finished).unwrap();
      }
    }
    AssetState::Finished => unreachable!(),
  }
}

fn init_assets(
  commands: &mut Commands,
  asset_handles: Res<AssetHandles>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  scene_spawner: ResMut<SceneSpawner>,
  mesh_handles: Query<&Handle<Mesh>>,
  mut debug_cube: ResMut<DebugCube>,
) {
  debug_cube.0 = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));

  for instance in &asset_handles.instances {
    let _errs = scene_spawner
      .iter_instance_entities(*instance)
      .unwrap()
      .for_each(|entity| {
        let mesh_handle = if let Ok(h) = mesh_handles.get(entity) {
          h
        } else {
          return;
        };

        let mesh = meshes.get(mesh_handle).unwrap();
        let mesh_wrapper = MeshWrapper::new(mesh, "Vertex_Normal", "Vertex_Position");
        let position = Isometry3::translation(0., 3., 0.);
        mesh_wrapper
          .build_collider(commands, entity, position, debug_cube.0.clone())
          .unwrap();
      });
  }

  /*
   * Ground
   */
  let ground_size = 200.1;
  let ground_height = 1.0;
  let extents = Vec3::new(0.5*ground_size, 0.5*ground_height, 0.5*ground_size);

  let rigid_body = RigidBodyBuilder::new_static().translation(0.0, -0.5 * ground_height, 0.0);
  let collider = ColliderBuilder::cuboid(extents.x, extents.y, extents.z);
  let color = Color::rgb(
    0xF3 as f32 / 255.0,
    0xD9 as f32 / 255.0,
    0xB1 as f32 / 255.0,
  );
  let pbr = PbrBundle {
    mesh: debug_cube.0.clone(),
    transform: Transform::from_scale(extents),
    material: materials.add(color.into()),
    ..Default::default()
  };
  commands.spawn((rigid_body, collider));
  commands.with_bundle(pbr);

  /*
   * Box
   */
  commands.spawn((
    RigidBodyBuilder::new_dynamic().translation(0., 7., 0.),
    ColliderBuilder::cuboid(1., 1., 1.).density(1.0),
  ));
  commands.with_bundle(PbrBundle {
    mesh: debug_cube.0.clone(),
    material: materials.add(color.into()),
    ..Default::default()
  });
}

#[derive(Default)]
pub struct DebugCube(pub Handle<Mesh>);

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<AssetHandles>()
      .init_resource::<DebugCube>()
      .add_resource(State::new(AssetState::Start))
      .add_stage_after(stage::UPDATE, "assets", StateStage::<AssetState>::default())
      .on_state_enter("assets", AssetState::Start, load_assets.system())
      .on_state_update("assets", AssetState::Loading, load_assets.system())
      .on_state_update("assets", AssetState::Spawning, load_assets.system())
      .on_state_enter("assets", AssetState::Finished, init_assets.system());
  }
}
