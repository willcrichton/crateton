use crate::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy_rapier3d::prelude::*;

use std::borrow::Cow;

pub struct MeshWrapper<'a> {
  mesh: &'a Mesh,
  normal_attribute: String,
  position_attribute: String,
}

impl<'a> MeshWrapper<'a> {
  pub fn new(
    mesh: &'a Mesh,
    position_attribute: impl Into<String>,
    normal_attribute: impl Into<String>,
  ) -> MeshWrapper<'a> {
    MeshWrapper {
      mesh,
      normal_attribute: normal_attribute.into(),
      position_attribute: position_attribute.into(),
    }
  }

  pub fn get_attribute(&self, name: impl Into<Cow<'static, str>>) -> Vec<Point<f32>> {
    let name = name.into();
    let attr = self
      .mesh
      .attribute(name.clone())
      .expect(&format!("invalid attribute name {}", name));
    match attr {
      VertexAttributeValues::Float32x3(v) => v
        .iter()
        .map(|p| point![p[0], p[1], p[2]])
        .collect::<Vec<_>>(),
      _ => unimplemented!(),
    }
  }

  pub fn vertices(&self) -> Vec<Point<f32>> {
    self.get_attribute(self.position_attribute.clone())
  }

  pub fn indices(&self) -> Vec<[u32; 3]> {
    match self.mesh.indices().as_ref().unwrap() {
      Indices::U32(indices) => indices
        .chunks(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect::<Vec<_>>(),
      _ => unimplemented!(),
    }
  }
}
