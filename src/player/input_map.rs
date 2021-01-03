use bevy::input::keyboard::KeyCode;

pub struct InputMap {
  pub key_forward: KeyCode,
  pub key_backward: KeyCode,
  pub key_left: KeyCode,
  pub key_right: KeyCode,
  pub key_jump: KeyCode,
  pub key_run: KeyCode,
  pub key_crouch: KeyCode,
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
      invert_y: false,
    }
  }
}
