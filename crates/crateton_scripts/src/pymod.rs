use rustpython_vm::pymodule;

#[pymodule]
pub mod crateton_pymod {
  use bevy::prelude::*;
  use rustpython_derive::{pyclass, pyimpl, pymodule};
  use rustpython_vm::{InitParameter, PySettings, VirtualMachine, builtins::{PyFloat, PyList, PyStr, PyStrRef, PyTypeRef}, compile, pyobject::{
      ItemProtocol, PyClassImpl, PyObjectRef, PyRef, PyResult, PyValue, StaticType, TryIntoRef,
    }};
  use std::{
    fmt, mem,
    sync::{Arc, Mutex},
  };

  macro_rules! pyvalue_impl {
    ($id:ident) => {
      impl PyValue for $id {
        fn class(_vm: &VirtualMachine) -> &PyTypeRef {
          Self::static_type()
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
        .map_err(|_| {
          vm.new_lookup_error(format!("Entity {:?} does not have Transform", self.entity))
        })
    }
  }

  #[derive(Default)]
  struct CWorldInner {
    world: World,
    resources: Resources,
  }

  #[pyattr]
  #[pyclass(name, module = "crateton")]
  pub struct CWorld {
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
    pub fn new(world: World, resources: Resources) -> Self {
      CWorld {
        inner: Mutex::new(CWorldInner { world, resources }),
      }
    }

    pub fn extract(&self) -> (World, Resources) {
      let mut inner = self.inner.lock().unwrap();
      let CWorldInner { world, resources } = mem::replace(&mut *inner, Default::default());
      (world, resources)
    }

    #[pymethod]
    fn entity_with_name(&self, name: PyStrRef, vm: &VirtualMachine) -> PyResult<CEntity> {
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

  #[pyattr]
  #[pyclass(name, module = "crateton")]
  pub struct CStdout {}
  pyvalue_impl!(CStdout);

  impl fmt::Debug for CStdout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str("CStdout")
    }
  }

  #[pyimpl]
  impl CStdout {
    #[pymethod]
    fn write(&self, data: PyStrRef, vm: &VirtualMachine) {
      info!("{}", data.as_ref());
    }

    #[pymethod]
    fn flush(&self) {}
  }
}
