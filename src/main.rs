use bevy::prelude::*;

mod assets;
mod map;
mod math;
mod physics;
mod player;
mod prelude;
mod shaders;
mod tools;
mod ui;
mod utils;

fn main() {
  let mut app = App::build();
  app
    .add_resource(Msaa { samples: 4 })
    .add_resource(WindowDescriptor {
      cursor_locked: true,
      cursor_visible: false,
      vsync: false,
      ..Default::default()
    })
    // Bevy core plugins
    .add_plugins(DefaultPlugins)
    // Internal plugins
    .add_plugin(assets::AssetsPlugin)
    .add_plugin(physics::PhysicsPlugin)
    .add_plugin(player::PlayerControllerPlugin)
    .add_plugin(tools::ToolPlugin)
    .add_plugin(shaders::ShadersPlugin)
    .add_plugin(map::MapPlugin)
    .add_plugin(ui::UiPlugin)
    // External plugins
    .add_plugin(bevy_rapier3d::physics::RapierPhysicsPlugin)
    .add_plugin(bevy_world_visualizer::WorldVisualizerPlugin);

  #[cfg(target_arch = "wasm32")]
  app.add_plugin(bevy_webgl2::WebGL2Plugin);

  app.run();
}
