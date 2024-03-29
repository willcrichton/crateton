use rustpython_vm::pymodule;

pub struct ScriptOutputEvent {
  pub output: String,
}

#[pymodule]
pub mod crateton_pymod {
  use super::ScriptOutputEvent;
  use crate::prelude::*;
  use rustpython_vm::{
    builtins::{PyFloat, PyList, PyStrRef, PyTypeRef},
    pyclass, pyimpl, ItemProtocol, PyRef, PyResult, PyValue, StaticType, TryIntoRef,
    VirtualMachine,
  };
  use std::{fmt, ptr::NonNull};

  macro_rules! pyvalue_impl {
    ($id:ident) => {
      impl PyValue for $id {
        fn class(_vm: &VirtualMachine) -> &PyTypeRef {
          Self::static_type()
        }
      }
    };
  }

  macro_rules! debug_impl {
    ($id:ident) => {
      impl fmt::Debug for $id {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          f.write_str(stringify!($id))
        }
      }
    };
  }

  #[pyattr]
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

  #[pyattr]
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
      CVec3 {
        vec: self.transform.translation.clone(),
      }
    }
  }

  #[pyattr]
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
      CWorld::fetch(vm)
        .world()
        .get::<Transform>(self.entity)
        .map(|transform| CTransform {
          transform: *transform,
        })
        .ok_or_else(|| {
          vm.new_lookup_error(format!("Entity {:?} does not have Transform", self.entity))
        })
    }
  }

  #[pyattr]
  #[pyclass(name, module = "crateton")]
  pub struct CWorld(NonNull<World>);
  pyvalue_impl!(CWorld);
  debug_impl!(CWorld);

  #[pyimpl]
  impl CWorld {
    pub fn fetch(vm: &VirtualMachine) -> PyRef<Self> {
      vm.current_globals()
        .get_item("world", vm)
        .unwrap()
        .try_into_ref(vm)
        .unwrap()
    }

    pub fn new(world: &mut World) -> Self {
      CWorld(NonNull::new(world).unwrap())
    }

    fn world(&self) -> &World {
      unsafe { self.0.as_ref() }
    }

    fn world_mut(&self) -> &mut World {
      unsafe { &mut *self.0.as_ptr() }
    }

    #[pymethod]
    fn entity_with_name(&self, name: PyStrRef, vm: &VirtualMachine) -> PyResult<CEntity> {
      let name = name.as_ref();
      self
        .world_mut()
        .query::<(Entity, &Name)>()
        .iter(self.world())
        .find(|(_, name_component)| name == name_component.as_str())
        .map(|(entity, _)| CEntity { entity })
        .ok_or_else(|| vm.new_lookup_error(format!("Name {} does not exist", name)))
    }
  }

  #[pyattr]
  #[pyclass(name, module = "crateton")]
  pub struct CStdout {}
  pyvalue_impl!(CStdout);
  debug_impl!(CStdout);

  #[pyimpl]
  impl CStdout {
    #[pymethod]
    fn write(&self, data: PyStrRef, vm: &VirtualMachine) {
      let world = CWorld::fetch(vm);
      let mut events = world
        .world_mut()
        .get_resource_mut::<Events<ScriptOutputEvent>>()
        .unwrap();
      events.send(ScriptOutputEvent {
        output: data.as_ref().to_owned(),
      });
    }

    #[pymethod]
    fn flush(&self) {}
  }
}
