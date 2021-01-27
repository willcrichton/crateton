use bevy::input::keyboard::KeyCode;

// TODO: modularize this so each component registers its own keys
pub struct InputMap {
  pub key_forward: KeyCode,
  pub key_backward: KeyCode,
  pub key_left: KeyCode,
  pub key_right: KeyCode,
  pub key_jump: KeyCode,
  pub key_run: KeyCode,
  pub key_crouch: KeyCode,

  pub key_toggle_camera_view: KeyCode,
  pub key_toggle_fly: KeyCode,
  pub key_show_ui: KeyCode,
  pub key_toggle_world_visualizer: KeyCode,
  pub key_rotate_toolgun: KeyCode,
  pub key_lock_rotation: KeyCode,

  pub invert_y: bool,
}

impl Default for InputMap {
  fn default() -> Self {
    Self {
      key_forward: KeyCode::W,
      key_backward: KeyCode::S,
      key_left: KeyCode::A,
      key_right: KeyCode::D,
      key_jump: KeyCode::Back,
      key_run: KeyCode::LShift,
      key_crouch: KeyCode::LControl,
      key_toggle_camera_view: KeyCode::V,
      key_toggle_fly: KeyCode::F,
      key_show_ui: KeyCode::Tab,
      key_toggle_world_visualizer: KeyCode::LAlt,
      key_rotate_toolgun: KeyCode::E,
      key_lock_rotation: KeyCode::LShift,
      invert_y: false,
    }
  }
}
