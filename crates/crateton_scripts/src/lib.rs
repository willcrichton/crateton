use bevy::prelude::*;
use rustpython_compiler as compiler;
use rustpython_derive::pyclass;
use rustpython_vm as vm;

#[pyclass(name, module = "crateton")]
struct CWorld {
  world: World,
  resources: Resources,
}

const SCRIPT: &'static str = r#"
print("Hi");
# print(world.entity_with_name("player body").transform().position().to_list())
"#;

fn run_scripts(world: &mut World, resources: &mut Resources) {
  let interpreter = resources.get_thread_local_mut::<vm::Interpreter>().unwrap();
  interpreter.enter(|vm| {
    let scope = vm.new_scope_with_builtins();
    //scope.globals.set_item("")
    let code_obj = vm
      .compile(SCRIPT, compiler::Mode::Exec, "<embedded>".to_owned())
      .unwrap();
    vm.run_code_obj(code_obj, scope.clone()).unwrap();
  });
}

pub struct ScriptsPlugin;

impl Plugin for ScriptsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_thread_local_resource::<vm::Interpreter>()
      //.add_startup_system(setup_scripts.system())
      .add_system(run_scripts.system());
  }
}
