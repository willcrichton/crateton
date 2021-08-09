use crate::prelude::*;
use std::marker::PhantomData;

use bevy::app::ManualEventReader;
use rustpython_vm::{
  self as vm, compile::Mode, InitParameter, IntoPyObject, ItemProtocol, PySettings, PyValue,
};
use vm::{builtins::PyNone, scope::Scope, Interpreter};

use pymod::{
  crateton_pymod::{CStdout, CWorld},
  ScriptOutputEvent,
};

pub mod pymod;

// const SCRIPT: &'static str = r#"
// # print(world.entity_with_name("player body").transform().position().to_list())
// "#;

struct PyInterpreter {
  interpreter: Interpreter,
  scope: Scope,
}

pub struct RunScriptEvent {
  pub code: String,
}

#[derive(Default)]
struct RunScriptEventReader(ManualEventReader<RunScriptEvent>);

fn run_scripts(world: &mut World) {
  let (py, mut event_reader, events) = unsafe {
    (
      world
        .get_non_send_resource_unchecked_mut::<PyInterpreter>()
        .unwrap(),
      world
        .get_resource_unchecked_mut::<RunScriptEventReader>()
        .unwrap(),
      world.get_resource::<Events<RunScriptEvent>>().unwrap(),
    )
  };

  py.interpreter.enter(|vm| {
    let run_code = |code: &str| -> anyhow::Result<()> {
      let code_obj = vm.compile(code, Mode::Exec, "<embedded>".to_owned())?;
      let output = vm.run_code_obj(code_obj, py.scope.clone());
      match output {
        Ok(_) => Ok(()),
        Err(exc) => {
          let mut error_text = String::new();
          vm::exceptions::write_exception(&mut error_text, vm, &exc).unwrap();
          Err(anyhow::Error::msg(error_text))
        }
      }
    };

    for event in event_reader.0.iter(&events) {
      if let Err(e) = run_code(&event.code) {
        warn!("Python error: {}", e);
      }
    }
  });
}

fn create_interpreter(world: &mut World) {
  let module_name = "crateton";
  let interpreter = vm::Interpreter::new_with_init(PySettings::default(), |vm| {
    vm.add_native_module(
      module_name.to_owned(),
      Box::new(pymod::crateton_pymod::make_module),
    );

    InitParameter::Internal
  });

  let scope = interpreter.enter(|vm| {
    // Make sure crateton is imported so constructors are initialized, ie cworld.into_ref doesn't panic
    vm.import(module_name, None, 0).unwrap();

    let stdout = (CStdout {}).into_ref(vm);
    vm.set_attr(&vm.sys_module, "stdout", stdout).unwrap();

    let cworld = CWorld::new(world);
    let cworld = cworld.into_ref(vm);

    let scope = vm.new_scope_with_builtins();
    scope
      .globals
      .set_item("world", cworld.clone().into(), vm)
      .unwrap();

    // Reset SIGINT handler to default so Ctrl-C exits application instead of getting caught by Python
    const RESET_SIGINT: &'static str = r#"
import signal
signal.signal(signal.SIGINT, signal.SIG_DFL)
    "#;
    if let Err(exc) = vm.run_code_obj(
      vm.compile(RESET_SIGINT, Mode::Exec, "<embedded>".to_owned())
        .unwrap(),
      scope.clone(),
    ) {
      vm::exceptions::print_exception(vm, exc);
    }

    scope
  });

  world.insert_non_send(PyInterpreter { interpreter, scope });
}

pub struct ScriptsPlugin;
impl Plugin for ScriptsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_startup_system(create_interpreter.exclusive_system())
      .add_system(run_scripts.exclusive_system())
      .init_resource::<RunScriptEventReader>()
      .add_event::<RunScriptEvent>()
      .add_event::<ScriptOutputEvent>();
  }
}
