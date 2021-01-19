use bevy::prelude::*;
use bevy_rapier3d::physics::RapierPhysicsPlugin;

mod assets;
mod math;
mod physics;
mod player;
mod prelude;
mod shaders;
mod tools;
mod map;

fn main() {
  let mut app = App::build();
  app
    .add_resource(Msaa { samples: 4})
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
    .add_plugin(map::MapPlugin);

  #[cfg(target_arch = "wasm32")]
  app.add_plugin(bevy_webgl2::WebGL2Plugin);

  app.run();
}