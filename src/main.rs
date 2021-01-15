use bevy::prelude::*;
use bevy_rapier3d::physics::RapierPhysicsPlugin;

mod assets;
mod math;
mod physics;
mod player;
mod prelude;
mod shaders;
mod tools;

fn main() {
  let mut app = App::build();
  app
    .add_resource(Msaa::default())
    .add_resource(WindowDescriptor {
      cursor_locked: true,
      cursor_visible: false,
      ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin)
    //.add_plugin(bevy_rapier3d::render::RapierRenderPlugin)
    .add_plugin(assets::AssetsPlugin)
    .add_plugin(physics::PhysicsPlugin)
    .add_plugin(player::PlayerControllerPlugin)
    .add_plugin(tools::ToolPlugin)
    .add_plugin(shaders::ShadersPlugin)
    .add_startup_system(setup_graphics.system());

  #[cfg(target_arch = "wasm32")]
  app.add_plugin(bevy_webgl2::WebGL2Plugin);

  app.run();
}

fn setup_graphics(commands: &mut Commands) {
  commands.spawn(LightBundle {
    transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
    ..Default::default()
  });
}
