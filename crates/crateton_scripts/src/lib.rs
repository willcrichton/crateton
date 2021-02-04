use std::{
  fmt, mem,
  sync::{Arc, Mutex},
};

use bevy::prelude::*;
use rustpython_vm::{
  self as vm,
  builtins::{PyFloat, PyList, PyStr, PyTypeRef},
  compile, pyclass, pyimpl, pymodule,
  pyobject::{
    ItemProtocol, PyClassImpl, PyObjectRef, PyRef, PyResult, PyValue, StaticType, TryIntoRef,
  },
  InitParameter, PySettings, VirtualMachine,
};
use vm::builtins::PyModule;

use pymod::crateton_pymod::{CStdout, CWorld};

mod pymod;

const SCRIPT: &'static str = r#"
print(world.entity_with_name("player body").transform().position().to_list())
"#;

fn run_scripts(world: &mut World, resources: &mut Resources) {
  let interpreter = resources
    .get_thread_local_mut::<Arc<vm::Interpreter>>()
    .unwrap()
    .clone();
  interpreter.enter(|vm| {
    take_mut::scoped::scope(|scope| {
      let (world, world_hole) = scope.take(world);
      let (resources, resources_hole) = scope.take(resources);

      let cworld = CWorld::new(world, resources);
      let cworld = cworld.into_ref(vm);

      let scope = vm.new_scope_with_builtins();
      scope
        .globals
        .set_item("world", cworld.clone().into(), vm)
        .unwrap();

      let code_obj = vm
        .compile(SCRIPT, compile::Mode::Exec, "<embedded>".to_owned())
        .unwrap();
      if let Err(exc) = vm.run_code_obj(code_obj, scope.clone()) {
        let mut error_text = Vec::new();
        vm::exceptions::write_exception(&mut error_text, vm, &exc).unwrap();
        warn!("Python error: {}", String::from_utf8(error_text).unwrap());
      }

      let (world, resources) = cworld.extract();
      world_hole.fill(world);
      resources_hole.fill(resources);
    });
  });
}

fn create_interpreter(_world: &mut World, resources: &mut Resources) {
  let vm = vm::Interpreter::new_with_init(PySettings::default(), |vm| {
    vm.add_native_module(
      "crateton".to_owned(),
      Box::new(pymod::crateton_pymod::make_module),
    );

    InitParameter::Internal
  });

  vm.enter(|vm| {
    // Make sure crateton is imported so constructors are initialized
    vm.import("crateton", None, 0).unwrap();

    let stdout = (CStdout {}).into_ref(vm);
    vm.set_attr(&vm.sys_module, "stdout", stdout).unwrap();
  });

  resources.insert_thread_local(Arc::new(vm));
}

pub struct ScriptsPlugin;

impl Plugin for ScriptsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_startup_system(create_interpreter.system())
      .add_system(run_scripts.system());
  }
}
