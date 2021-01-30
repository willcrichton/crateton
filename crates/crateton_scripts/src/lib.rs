use bevy::prelude::*;

use rustpython_compiler as compiler;
use rustpython_vm as vm;

use vm::pyobject::PyResult;

fn setup_scripts() {
  vm::Interpreter::default()
    .enter::<_, PyResult<()>>(|vm| {
      let scope = vm.new_scope_with_builtins();

      let code_obj = vm
        .compile(
          r#"print("Hello World!")"#,
          compiler::compile::Mode::Exec,
          "<embedded>".to_owned(),
        )
        .map_err(|err| vm.new_syntax_error(&err))?;

      vm.run_code_obj(code_obj, scope)?;

      Ok(())
    })
    .unwrap();
}

pub struct ScriptsPlugin;

impl Plugin for ScriptsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app.add_startup_system(setup_scripts.system());
  }
}
