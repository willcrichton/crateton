use bevy::{asset::LoadState, prelude::*, render::mesh::{VertexAttributeValues, Indices}, scene::InstanceId};
use bevy_rapier3d::physics::RapierPhysicsPlugin;
use rapier3d::math::Point;
use nalgebra::{Matrix3x1};
use rapier3d::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder};
//use crateton_core::physics::MeshExt;
use crateton_core::controls::{FlyCamera, FlyCameraPlugin};
use ncollide3d::{bounding_volume::{HasBoundingVolume, AABB}, shape::{TriMesh as NSTriMesh}, procedural::{IndexBuffer, TriMesh as NTrimesh}, transformation::hacd};

//mod hacd;

fn main() {
  App::build()
    .add_resource(Msaa::default())
    .add_resource(WindowDescriptor {
      width: 1280. * 2.,
      height: 720. * 2.,
      ..Default::default()
    })
    .init_resource::<AssetHandles>()
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin)
    .add_plugin(FlyCameraPlugin)
    .add_resource(State::new(AssetState::Loading))
    .add_stage_after(stage::UPDATE, "assets", StateStage::<AssetState>::default())
    .on_state_enter("assets", AssetState::Loading, load_assets.system())
    .on_state_update("assets", AssetState::Loading, check_assets.system())
    .on_state_update("assets", AssetState::Spawning, check_assets.system())
    .on_state_enter("assets", AssetState::Finished, init_assets.system())
    .add_startup_system(setup_graphics.system())
    .add_startup_system(crateton_scripts::setup_scripts.system())
    .run();
}
#[derive(Clone, Debug)]
enum AssetState {
  Loading,
  Spawning,
  Finished,
}

fn setup_graphics(commands: &mut Commands) {
  commands
    .spawn(LightBundle {
      transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
      ..Default::default()
    })
    .spawn(Camera3dBundle {
      transform: Transform::from_matrix(Mat4::face_toward(
        Vec3::new(0.0, 3.0, 10.0),
        Vec3::new(0.0, 3.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
      )),
      ..Default::default()
    })
    .with(FlyCamera::default());
}

#[derive(Default)]
struct AssetHandles {
  handles: Vec<Handle<Scene>>,
  instances: Vec<InstanceId>
}

fn load_assets(
  mut asset_handles: ResMut<AssetHandles>,
  asset_server: Res<AssetServer>,
) {
  asset_handles.handles = vec![asset_server.load("models/Monkey.gltf#Scene0")];
}

fn check_assets(
  mut state: ResMut<State<AssetState>>,
  mut asset_handles: ResMut<AssetHandles>,
  asset_server: Res<AssetServer>,
  mut scene_spawner: ResMut<SceneSpawner>,
  scenes: ResMut<Assets<Scene>>,
) {
  match state.current() {
    AssetState::Loading => {
      if let LoadState::Loaded =
        asset_server.get_group_load_state(asset_handles.handles.iter().map(|h| h.id))
      {
        asset_handles.instances = asset_handles.handles
          .iter()
          .map(|handle| {
            debug_assert!(scenes.get(handle).is_some(), "scene isn't properly loaded");
            scene_spawner.spawn(handle.clone())
          })
          .collect();
        state.set_next(AssetState::Spawning).unwrap();
      }
    },
    AssetState::Spawning => {
      if asset_handles.instances.iter().all(|inst| scene_spawner.instance_is_ready(*inst)) {
        state.set_next(AssetState::Finished).unwrap();
      }
    },
    AssetState::Finished => unreachable!()
  }
}
struct GltfAsset;

fn init_assets(
  commands: &mut Commands,
  asset_handles: Res<AssetHandles>,
  scenes: ResMut<Assets<Scene>>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  scene_spawner: ResMut<SceneSpawner>,
  mesh_handles: Query<&Handle<Mesh>>,
) {
  let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

  for (instance, handle) in asset_handles.instances.iter().zip(asset_handles.handles.iter()) {    
    let scene = scenes.get(handle).unwrap();    
    let _errs = scene_spawner.iter_instance_entities(*instance).unwrap().map(|entity| { 
      let mesh_handle = mesh_handles.get(entity).ok()?;
      let mesh = meshes.get(mesh_handle)?;

      let indices = match mesh.indices().as_ref().unwrap() {
        Indices::U32(indices) => indices
          .chunks(3)
          .map(|c| Point::new(c[0], c[1], c[2]))
          .collect::<Vec<_>>(),
        _ => unimplemented!(),
      };

      let get_attribute = |name| {
        let attr = mesh.attribute(name).unwrap();
        match attr {
          VertexAttributeValues::Float3(v) => v
            .iter()
            .map(|p| Point::new(p[0], p[1], p[2]))
            .collect::<Vec<_>>(),
          _ => unimplemented!(),
        }
      };

      let vertices = get_attribute("Vertex_Position");
      let normals = get_attribute("Vertex_Normal");

      let trimesh = NTrimesh::new(
        vertices.clone(),
        Some(normals.clone().into_iter().map(|p| Matrix3x1::from_iterator(p.iter().cloned())).collect()),
        None,
        Some(IndexBuffer::Unified(indices.clone())),
      );
      
      let (decomp, _partition) = hacd(trimesh, 0.03, 0);
      let colliders = decomp.into_iter()
        .map(|trimesh| {
          let aabb: AABB<_> = NSTriMesh::from(trimesh).local_bounding_volume();
          let center = aabb.center();
          let extents = aabb.half_extents();
          let pbr = PbrBundle {
            mesh: cube.clone(),
            transform: Transform::from_scale(Vec3::new(extents[0]*2., extents[1]*2., extents[2]*2.)),
            ..Default::default()
          };
          let collider = ColliderBuilder::cuboid(extents[0], extents[1], extents[2])
            .translation(center.x, center.y, center.z)
            .density(1.0)
            .build();
          (pbr, collider)
        })
        .collect::<Vec<_>>();

      let rigid_body = RigidBodyBuilder::new_static().translation(0., 0., 0.);
      //let pbr = PbrBundle { mesh: mesh_handle.clone(), ..Default::default() };

      commands.set_current_entity(entity);      
      commands.with(rigid_body);
      //commands.with_bundle(pbr);

      let entity = commands.current_entity().unwrap();
      for (pbr, collider) in colliders {
        commands.spawn((Parent(entity), collider));
        commands.with_bundle(pbr);
      }
      Some(())
    }).collect::<Vec<_>>();
  }

  /*
   * Ground
   */
  let ground_size = 200.1;
  let ground_height = 0.1;

  let rigid_body = RigidBodyBuilder::new_static().translation(0.0, -ground_height, 0.0);
  let collider = ColliderBuilder::cuboid(ground_size, ground_height, ground_size);
  let color = Color::rgb(
    0xF3 as f32 / 255.0,
    0xD9 as f32 / 255.0,
    0xB1 as f32 / 255.0,
  );
  let pbr = PbrBundle {
    mesh: cube,
    transform: Transform::from_scale(Vec3::new(ground_size, ground_height, ground_size)),
    material: materials.add(color.into()),
    ..Default::default()
  };
  commands.spawn((rigid_body, collider));
  commands.with_bundle(pbr);

  /*
   * Monkey
   */

  // for (_, mesh) in meshes.iter() {
  //   println!("{:?}", mesh);
  // }

  // let monkey_handle = asset_server.load_sync(&mut meshes, "assets/models/Monkey.gltf").unwrap();
  // let monkey_body = RigidBodyBuilder::new_static().translation(2., 4., 0.);
  // let monkey_mesh = meshes.get(&monkey_handle).unwrap();
  // let coords = monkey_mesh.vertices().to_vec();
  // let indices = IndexBuffer::Unified(monkey_mesh.indices().to_Vec());

  // let trimesh = NTrimesh::new(coords, None, None, Some(indices));
  // hacd(trimesh, 0.03, 0);

  //let monkey_collider = monkey_mesh.build_collider("Vertex_Position").unwrap().density(1.0);

  // let material = materials.add(StandardMaterial {
  //     albedo: Color::rgb(0.5, 0.4, 0.3),
  //     ..Default::default()
  // });
  // let monkey_pbr = PbrBundle { mesh: monkey_handle, material, ..Default::default() };
  //commands.spawn((monkey_body, monkey_collider));
  //commands.with_bundle(monkey_pbr);

  /*
   * Box
   */
  commands.spawn((
    RigidBodyBuilder::new_dynamic().translation(0., 7., 0.),
    ColliderBuilder::cuboid(1., 1., 1.).density(1.0),
  ));
  commands.with_bundle(PbrBundle {
    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    material: materials.add(color.into()),
    ..Default::default()
  });
}
