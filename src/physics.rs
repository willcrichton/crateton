use bevy::{
  prelude::*,
  render::mesh::{Indices, VertexAttributeValues},
};
use bevy_rapier3d::{
  na::{Isometry3, Matrix3x1},
  rapier::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder, math::Point},
};
use ncollide3d::{
  bounding_volume::{HasBoundingVolume, AABB},
  procedural::{IndexBuffer, TriMesh as NTrimesh},
  shape::TriMesh as NSTriMesh,
};
use std::borrow::Cow;

const DEBUG: bool = false;

pub struct MeshWrapper<'a> {
  mesh: &'a Mesh,
  normal_attribute: String,
  position_attribute: String,
}

impl<'a> MeshWrapper<'a> {
  pub fn new(
    mesh: &'a Mesh,
    normal_attribute: impl Into<String>,
    position_attribute: impl Into<String>,
  ) -> MeshWrapper<'a> {
    MeshWrapper {
      mesh,
      normal_attribute: normal_attribute.into(),
      position_attribute: position_attribute.into(),
    }
  }

  pub fn build_collider(
    &self,
    commands: &mut Commands,
    entity: Entity,
    position: Isometry3<f32>,
    debug_cube: Handle<Mesh>,
  ) -> Option<()> {
    let trimesh = self.to_ncollide_trimesh();
    let (decomp, _partition) = ncollide3d::transformation::hacd(trimesh, 0.03, 0);

    let colliders = decomp
      .into_iter()
      .map(|trimesh| {
        let aabb: AABB<_> = NSTriMesh::from(trimesh).local_bounding_volume();
        let center = aabb.center();
        let extents = aabb.half_extents();
        let pbr = PbrBundle {
          mesh: debug_cube.clone(),
          transform: Transform {
            scale: Vec3::from_slice_unaligned(extents.as_slice()),
            translation: Vec3::new(center.x, center.y, center.z),
            ..Default::default()
          },
          ..Default::default()
        };
        let collider = ColliderBuilder::cuboid(extents[0], extents[1], extents[2])
          .translation(center.x, center.y, center.z)
          .density(1.0);
        (pbr, collider)
      })
      .collect::<Vec<_>>();

    let rigid_body = RigidBodyBuilder::new_dynamic().position(position);

    commands.set_current_entity(entity);
    commands.with(rigid_body);

    for (pbr, collider) in colliders {
      commands.spawn((Parent(entity), tag_collider_with(collider, entity)));

      if DEBUG {
        commands.with_bundle(pbr);
      }
    }

    Some(())
  }

  fn get_attribute(&self, name: impl Into<Cow<'static, str>>) -> Vec<Point<f32>> {
    let name = name.into();
    let attr = self
      .mesh
      .attribute(name.clone())
      .expect(&format!("invalid attribute name {}", name));
    match attr {
      VertexAttributeValues::Float3(v) => v
        .iter()
        .map(|p| Point::new(p[0], p[1], p[2]))
        .collect::<Vec<_>>(),
      _ => unimplemented!(),
    }
  }

  fn to_ncollide_trimesh(&self) -> NTrimesh<f32> {
    let indices = match self.mesh.indices().as_ref().unwrap() {
      Indices::U32(indices) => indices
        .chunks(3)
        .map(|c| Point::new(c[0], c[1], c[2]))
        .collect::<Vec<_>>(),
      _ => unimplemented!(),
    };

    let vertices = self.get_attribute(self.position_attribute.clone());
    let normals = self.get_attribute(self.normal_attribute.clone());

    NTrimesh::new(
      vertices.clone(),
      Some(
        normals
          .clone()
          .into_iter()
          .map(|p| Matrix3x1::from_iterator(p.iter().cloned()))
          .collect(),
      ),
      None,
      Some(IndexBuffer::Unified(indices.clone())),
    )
  }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
  fn build(&self, _app: &mut AppBuilder) {
    //app.add_startup_system(init_physics.system());
    //app.add_system(player_system.system());
  }
}


pub fn tag_collider_with(collider: ColliderBuilder, entity: Entity) -> ColliderBuilder {
  collider.user_data(entity.to_bits() as u128)
}
pub fn tag_collider(commands: &mut Commands, collider: ColliderBuilder) -> ColliderBuilder {
  let entity = commands.current_entity().unwrap();
  tag_collider_with(collider, entity)
}
