use bevy::prelude::*;
use bevy_rapier3d::physics::RapierPhysicsPlugin;
use crateton_core::controls::FlyCamera;


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
    .add_plugin(crateton_core::controls::FlyCameraPlugin)
    .add_plugin(crateton_core::assets::AssetsPlugin)
    .add_plugin(crateton_scripts::ScriptsPlugin)
    .add_startup_system(setup_graphics.system())
    .run();
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

