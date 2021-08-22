use rg3d::core::wasm_bindgen::{self, prelude::*};
use std::panic;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn error(msg: String);

  type Error;

  #[wasm_bindgen(constructor)]
  fn new() -> Error;

  #[wasm_bindgen(structural, method, getter)]
  fn stack(error: &Error) -> String;
}

pub fn panic_hook(info: &panic::PanicInfo) {
  let mut msg = info.to_string();

  // Add the error stack to our message.
  //
  // This ensures that even if the `console` implementation doesn't
  // include stacks for `console.error`, the stack is still available
  // for the user. Additionally, Firefox's console tries to clean up
  // stack traces, and ruins Rust symbols in the process
  // (https://bugzilla.mozilla.org/show_bug.cgi?id=1519569) but since
  // it only touches the logged message's associated stack, and not
  // the message's contents, by including the stack in the message
  // contents we make sure it is available to the user.
  msg.push_str("\n\nStack:\n\n");
  let e = Error::new();
  let stack = e.stack();
  msg.push_str(&stack);

  // Safari's devtools, on the other hand, _do_ mess with logged
  // messages' contents, so we attempt to break their heuristics for
  // doing that by appending some whitespace.
  // https://github.com/rustwasm/console_error_panic_hook/issues/7
  msg.push_str("\n\n");

  // Finally, log the panic with `console.error`!
  error(msg);
}

#[wasm_bindgen]
pub fn main() {
  use std::sync::Once;
  static SET_HOOK: Once = Once::new();
  SET_HOOK.call_once(|| {
    panic::set_hook(Box::new(panic_hook));
  });

  rg3d::core::wasm_bindgen_futures::spawn_local(async {
    crate::run().await;
  });
}