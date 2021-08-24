use crateton::{prelude::*, *};

fn main() {
  let mut app = App::new();

  #[cfg(target_arch = "wasm32")]
  {
    let window = web_sys::window().unwrap();
    let width = window.inner_width().unwrap().as_f64().unwrap() as f32;
    let height = window.inner_height().unwrap().as_f64().unwrap() as f32;

    // Has to go before DefaultPlugins
    app.insert_resource(WindowDescriptor {
      canvas: Some("#game".to_string()),
      width,
      height,
      ..Default::default()
    });
  }

  app
    .insert_resource(Msaa { samples: 4 })
    .add_plugins(DefaultPlugins)
    .add_plugin(shaders::ShadersPlugin)
    .add_plugin(physics::PhysicsPlugin)
    .add_plugin(player::PlayerControllerPlugin)
    .add_plugin(tools::ToolPlugin)
    .add_plugin(map::MapPlugin)
    .add_plugin(ui::UiPlugin)
    .add_plugin(serde::SerdePlugin)
    .add_plugin(models::ModelsPlugin)
    .add_plugin(scripts::ScriptsPlugin);

  #[cfg(target_arch = "wasm32")]
  {
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
  }

  app.run();
}
