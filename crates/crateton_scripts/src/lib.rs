use bevy::prelude::*;
use rustpython_vm::{
  self as vm, compile,
  pyobject::{ItemProtocol, PyValue},
  InitParameter, PySettings,
};
use vm::{scope::Scope, Interpreter};

use pymod::crateton_pymod::{CStdout, CWorld};

mod pymod;

const SCRIPT: &'static str = r#"
print(world.entity_with_name("player body").transform().position().to_list())
"#;

struct PyInterpreter {
  interpreter: Interpreter,
  scope: Scope,
}

fn run_scripts(_world: &mut World, resources: &mut Resources) {
  let py = resources.get_thread_local_mut::<PyInterpreter>().unwrap();
  py.interpreter.enter(|vm| {
    let code_obj = vm
      .compile(SCRIPT, compile::Mode::Exec, "<embedded>".to_owned())
      .unwrap();
    if let Err(exc) = vm.run_code_obj(code_obj, py.scope.clone()) {
      let mut error_text = Vec::new();
      vm::exceptions::write_exception(&mut error_text, vm, &exc).unwrap();
      warn!("Python error: {}", String::from_utf8(error_text).unwrap());
    }
  });
}

fn create_interpreter(world: &mut World, resources: &mut Resources) {
  let interpreter = vm::Interpreter::new_with_init(PySettings::default(), |vm| {
    vm.add_native_module(
      "crateton".to_owned(),
      Box::new(pymod::crateton_pymod::make_module),
    );

    InitParameter::Internal
  });

  let scope = interpreter.enter(|vm| {
    // Make sure crateton is imported so constructors are initialized
    vm.import("crateton", None, 0).unwrap();

    let stdout = (CStdout {}).into_ref(vm);
    vm.set_attr(&vm.sys_module, "stdout", stdout).unwrap();

    let cworld = CWorld::new(world, resources);
    let cworld = cworld.into_ref(vm);

    let scope = vm.new_scope_with_builtins();
    scope
      .globals
      .set_item("world", cworld.clone().into(), vm)
      .unwrap();

    scope
  });

  resources.insert_thread_local(PyInterpreter { interpreter, scope });
}

pub struct ScriptsPlugin;

impl Plugin for ScriptsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_startup_system(create_interpreter.system())
      .add_system(run_scripts.system());
  }
}
