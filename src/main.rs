use bevy::prelude::*;
use bevy_rapier3d::physics::RapierPhysicsPlugin;
use rapier3d::{dynamics::RigidBodyBuilder, geometry::{ColliderBuilder}};
use crateton_core::physics::MeshExt;
use crateton_core::controls::{FlyCamera, FlyCameraPlugin};


#[derive(Debug)]
struct Test { x: i32 }

fn main() {
  println!("{:?}", Test { x: 0 });

  App::build()
    .add_resource(Msaa::default())
    .add_resource(WindowDescriptor {
      width: 1280*2,
      height: 720*2,
      ..Default::default()}
    )
    .add_default_plugins()
    .add_plugin(RapierPhysicsPlugin)
    .add_plugin(FlyCameraPlugin)
    .add_startup_system(setup_graphics.system())
    .add_startup_system(setup_physics.system())
    .add_startup_system(crateton_scripts::setup_scripts.system())
    .run();
}


fn setup_scripts(mut commands: Commands) {
  crateton_scripts::setup_scripts();
}

fn setup_graphics(mut commands: Commands) {
  commands
      .spawn(LightComponents {
          transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
          ..Default::default()
      })
      .spawn(Camera3dComponents {
          transform: Transform::new(Mat4::face_toward(
              Vec3::new(0.0, 3.0, 10.0),
              Vec3::new(0.0, 3.0, 0.0),
              Vec3::new(0.0, 1.0, 0.0),
          )),
          ..Default::default()
      })
     .with(FlyCamera::default());
}


pub fn setup_physics(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  asset_server.load_asset_folder("assets/models").unwrap();

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
  let pbr = PbrComponents {
    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    transform: Transform::from_non_uniform_scale(Vec3::new(ground_size, ground_height, ground_size)),
    material: materials.add(color.into()),
    ..Default::default()
  };
  commands.spawn((rigid_body, collider));
  commands.with_bundle(pbr);

  /*
   * Monkey
   */
  let monkey_handle = asset_server.load_sync(&mut meshes, "assets/models/Monkey.gltf").unwrap();
  let monkey_body = RigidBodyBuilder::new_static().translation(2., 4., 0.);
  let monkey_collider = meshes.get(&monkey_handle).unwrap().build_collider("Vertex_Position").unwrap().density(1.0);

  let material = materials.add(StandardMaterial {
      albedo: Color::rgb(0.5, 0.4, 0.3),
      ..Default::default()
  });
  let monkey_pbr = PbrComponents { mesh: monkey_handle, material, ..Default::default() };
  commands.spawn((monkey_body, monkey_collider));
  commands.with_bundle(monkey_pbr);

  /*
   * Box
   */
  commands.spawn((
    RigidBodyBuilder::new_dynamic().translation(0., 7., 0.),
    ColliderBuilder::cuboid(1., 1., 1.).density(1.0),
  ));
  commands.with_bundle(PbrComponents {
    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    material: materials.add(color.into()),
    ..Default::default()
  });
}
