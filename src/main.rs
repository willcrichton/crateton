use bevy::prelude::*;

mod json;
mod map;
mod math;
mod models;
mod physics;
mod player;
mod prelude;
mod shaders;
mod tools;
mod ui;
mod utils;

fn main() {
  let mut app = App::build();

  #[cfg(target_arch = "wasm32")]
  {
    let window = web_sys::window().unwrap();
    let width = window.inner_width().unwrap().as_f64().unwrap() as f32;
    let height = window.inner_height().unwrap().as_f64().unwrap() as f32;
    
    // Has to go before DefaultPlugins
    app.add_resource(WindowDescriptor {
      canvas: Some("#game".to_string()),
      width, height,
      ..Default::default()
    });
  }

  app
    .add_resource(Msaa { samples: 4 })
    // Bevy core plugins
    .add_plugins(DefaultPlugins)
    // Internal plugins
    .add_plugin(physics::PhysicsPlugin)
    .add_plugin(player::PlayerControllerPlugin)
    .add_plugin(tools::ToolPlugin)
    .add_plugin(shaders::ShadersPlugin)
    .add_plugin(map::MapPlugin)
    .add_plugin(ui::UiPlugin)
    .add_plugin(json::JsonPlugin)
    .add_plugin(models::ModelsPlugin);

  // External plugins
  // app.add_plugin(bevy_rapier3d::render::RapierRenderPlugin);

  #[cfg(target_arch = "wasm32")]
  {
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
  }

  app.add_system(capture_first_click.system());

  app.run();
}

fn capture_first_click(
  mut windows: ResMut<Windows>,
  mut done: Local<bool>,
  mouse_input: Res<Input<MouseButton>>,
) {
  if *done {
    return;
  }

  let window = windows.get_primary_mut().unwrap();
  if mouse_input.just_pressed(MouseButton::Left) {
    window.set_cursor_lock_mode(true);
    window.set_cursor_visibility(false);
    *done = true;
  }
}
