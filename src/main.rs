use bevy::prelude::*;
use bevy_rapier3d::physics::RapierPhysicsPlugin;

mod assets;
mod physics;
mod player;

fn main() {
  App::build()
    .add_resource(Msaa::default())
    .add_resource(WindowDescriptor {
      width: 1280. * 2.,
      height: 720. * 2.,
      ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin)
    //.add_plugin(bevy_rapier3d::render::RapierRenderPlugin)
    .add_plugin(assets::AssetsPlugin)
    .add_plugin(physics::PhysicsPlugin)
    //.add_plugin(crateton_scripts::ScriptsPlugin)
    .add_plugin(player::PlayerControllerPlugin)
    .add_startup_system(setup_graphics.system())
    .run();
}

fn setup_graphics(commands: &mut Commands) {
  commands.spawn(LightBundle {
    transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
    ..Default::default()
  });
  // .spawn(Camera3dBundle {
  //   transform: Transform::from_matrix(Mat4::face_toward(
  //     Vec3::new(0.0, 3.0, 10.0),
  //     Vec3::new(0.0, 3.0, 0.0),
  //     Vec3::new(0.0, 1.0, 0.0),
  //   )),
  //   ..Default::default()
  // });

  //let camera = commands.current_entity().unwrap();
  //crateton_core::physics::init_physics(commands, camera);
  //.with(FlyCamera::default());
}
