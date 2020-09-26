use bevy::prelude::*;

use bevy::render::mesh::VertexAttributeValues;
use rapier3d::{geometry::{ColliderBuilder}, math::Point};

pub trait MeshExt {
  fn build_collider(&self, position_attribute: &'static str) -> Option<ColliderBuilder>;
}

impl MeshExt for Mesh {
    fn build_collider(&self, position_attribute: &'static str) -> Option<ColliderBuilder> {
        let indices = self.indices.as_ref().unwrap()
            .chunks(3)
            .map(|c| Point::new(c[0], c[1], c[2]))
            .collect::<Vec<_>>();
        let attr = self.attributes.iter().find(|a| a.name == position_attribute)?;
        let vertices = match &attr.values {
            VertexAttributeValues::Float3(v) => v.iter().map(|p| Point::new(p[0], p[1], p[2])).collect::<Vec<_>>(),
            _ => unimplemented!()
        };
        Some(ColliderBuilder::trimesh(vertices, indices))
    }
}