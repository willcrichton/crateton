use std::{
  fmt, mem,
  sync::{Arc, Mutex},
};

use bevy::prelude::*;
use rustpython_compiler as compiler;
use rustpython_derive::{pyclass, pyimpl};
use rustpython_vm as vm;
use vm::{VirtualMachine, builtins::{PyFloat, PyList, PyStr, PyTypeRef}, pyobject::{
    ItemProtocol, PyClassImpl, PyObjectRef, PyRef, PyResult, PyValue, StaticType, TryIntoRef,
  }};

macro_rules! pyvalue_impl {
  ($id:ident) => {
    impl PyValue for $id {
      fn class(_vm: &VirtualMachine) -> &PyTypeRef {
        Self::static_type()
      }
    }
  };
}

#[pyclass(name, module = "crateton")]
#[derive(Debug)]
struct CVec3 {
  vec: Vec3,
}
pyvalue_impl!(CVec3);

#[pyimpl]
impl CVec3 {
  #[pymethod]
  fn to_list(&self, vm: &VirtualMachine) -> PyList {
    vec![self.vec.x, self.vec.y, self.vec.z]
      .into_iter()
      .map(|n| {
        let n: PyFloat = (n as f64).into();
        n.into_ref(vm).into()
      })
      .collect::<Vec<_>>()
      .into()
  }
}

#[pyclass(name, module = "crateton")]
#[derive(Debug)]
struct CTransform {
  transform: Transform,
}
pyvalue_impl!(CTransform);

#[pyimpl]
impl CTransform {
  #[pymethod]
  fn position(&self, _vm: &VirtualMachine) -> CVec3 {
    CVec3 { vec: self.transform.translation.clone() }
  }
}

#[pyclass(name, module = "crateton")]
#[derive(Debug)]
struct CEntity {
  entity: Entity,
}
pyvalue_impl!(CEntity);

#[pyimpl]
impl CEntity {
  #[pymethod]
  fn transform(&self, vm: &VirtualMachine) -> PyResult<CTransform> {
    let world: PyRef<CWorld> = vm
      .current_globals()
      .get_item("world", vm)?
      .try_into_ref(vm)?;
    let world = world.inner.lock().unwrap();
    world
      .world
      .get::<Transform>(self.entity)
      .map(|transform| CTransform {
        transform: *transform,
      })
      .map_err(|_| vm.new_lookup_error(format!("Entity {:?} does not have Transform", self.entity)))
  }
}

#[derive(Default)]
struct CWorldInner {
  world: World,
  resources: Resources,
}

#[pyclass(name, module = "crateton")]
struct CWorld {
  inner: Mutex<CWorldInner>,
}
pyvalue_impl!(CWorld);

impl fmt::Debug for CWorld {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("CWorld")
  }
}

#[pyimpl]
impl CWorld {
  #[pymethod]
  fn entity_with_name(&self, name: PyObjectRef, vm: &VirtualMachine) -> PyResult<CEntity> {
    let name: PyRef<PyStr> = name.try_into_ref(vm)?;
    let name = name.as_ref();
    let inner = self.inner.lock().unwrap();
    inner
      .world
      .query::<(Entity, &Name)>()
      .find(|(_, name_component)| name == name_component.as_str())
      .map(|(entity, _)| CEntity { entity })
      .ok_or_else(|| vm.new_lookup_error(format!("Name {} does not exist", name)))
  }
}

const SCRIPT: &'static str = r#"
print(world.entity_with_name("player body").transform().position().to_list());
# print(world.entity_with_name("player body").transform().position().to_list())
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

      CVec3::make_class(&vm.ctx);
      CTransform::make_class(&vm.ctx);
      CEntity::make_class(&vm.ctx);
      CWorld::make_class(&vm.ctx);

      let cworld = CWorld {
        inner: Mutex::new(CWorldInner { world, resources }),
      };
      let cworld = cworld.into_ref(vm);

      let scope = vm.new_scope_with_builtins();
      scope
        .globals
        .set_item("world", cworld.clone().into(), vm)
        .unwrap();

      let code_obj = vm
        .compile(SCRIPT, compiler::Mode::Exec, "<embedded>".to_owned())
        .unwrap();
      if let Err(e) = vm.run_code_obj(code_obj, scope.clone()) {
        vm::exceptions::print_exception(vm, e);
      }

      let mut inner = cworld.inner.lock().unwrap();
      let CWorldInner { world, resources } = mem::replace(&mut *inner, Default::default());
      world_hole.fill(world);
      resources_hole.fill(resources);
    });
  });
}

pub struct ScriptsPlugin;

impl Plugin for ScriptsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_thread_local_resource::<Arc<vm::Interpreter>>()
      //.add_startup_system(setup_scripts.system())
      .add_system(run_scripts.system());
  }
}
