use crate::{utils, prelude::*};
use bevy::{
  prelude::*,
  render::mesh::{Indices, VertexAttributeValues},
};
use bevy_rapier3d::{
  na::{Matrix3x1, Point3, Vector3},
  rapier::{
    dynamics::{BodyStatus, RigidBody, RigidBodyBuilder},
    geometry::ColliderBuilder,
    math::Point,
  },
};
use ncollide3d::{
  bounding_volume::{HasBoundingVolume, AABB},
  procedural::{IndexBuffer, TriMesh as NTrimesh},
  shape::TriMesh as NSTriMesh,
  utils::tetrahedron_signed_volume,
};
use std::borrow::Cow;

pub struct MeshWrapper<'a> {
  mesh: &'a Mesh,
  normal_attribute: String,
  position_attribute: String,
  scale: Vector3<f32>,
}

const HACD_ERROR: f32 = 0.03;
const HACD_MIN_COMPONENTS: usize = 0;
const HACD_EPSILON: f32 = 0.01;

impl<'a> MeshWrapper<'a> {
  pub fn new(
    mesh: &'a Mesh,
    position_attribute: impl Into<String>,
    normal_attribute: impl Into<String>,
    scale: Vector3<f32>,
  ) -> MeshWrapper<'a> {
    MeshWrapper {
      mesh,
      normal_attribute: normal_attribute.into(),
      position_attribute: position_attribute.into(),
      scale,
    }
  }

  fn build_approx_collider(
    &self,
    commands: &mut Commands,
    entity: Entity,
    mass: f32,
    debug_cube: Option<Handle<Mesh>>,
  ) -> Option<()> {
    let trimesh = self.to_ncollide_trimesh();
    let (decomp, _partition) =
      ncollide3d::transformation::hacd(trimesh, HACD_ERROR, HACD_MIN_COMPONENTS);
    info!("after hacd {}", decomp.len());

    let num_colliders = decomp.len();
    let colliders = decomp
      .into_iter()
      .map(|trimesh| {
        // Get AABB from trimesh
        let aabb: AABB<_> = NSTriMesh::from(trimesh).local_bounding_volume();

        // If HACD computes an AABB with zero extent in any dimension, collider will end up with zero volume
        // and hence zero mass. So we add a small epsilon
        let half_extents = aabb.half_extents().add_scalar(HACD_EPSILON);
        let aabb = AABB::from_half_extents(aabb.center(), half_extents);
        let center = aabb.center();

        let target_mass = mass / (num_colliders as f32);
        let density = target_mass / aabb.volume();

        let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
          .translation(center.x, center.y, center.z)
          .density(density);

        let pbr = debug_cube.as_ref().map(|cube| PbrBundle {
          mesh: cube.clone(),
          transform: Transform {
            scale: half_extents.to_glam_vec3(),
            translation: center.to_glam_vec3(),
            ..Default::default()
          },
          ..Default::default()
        });

        (collider, pbr)
      })
      .collect::<Vec<_>>();

    for (collider, pbr) in colliders {
      commands.spawn((Parent(entity), collider, Name::new("collider child")));

      if let Some(pbr) = pbr {
        commands.with_bundle(pbr);
      }
    }

    Some(())
  }

  fn build_exact_collider(&self, commands: &mut Commands, entity: Entity, mass: f32) -> Option<()> {
    let vertices = self.vertices();
    let indices = self.indices();

    // https://stackoverflow.com/questions/1406029/how-to-calculate-the-volume-of-a-3d-mesh-object-the-surface-of-which-is-made-up
    let volume = {
      indices
        .iter()
        .map(|idx| {
          tetrahedron_signed_volume(
            &vertices[idx.x as usize],
            &vertices[idx.y as usize],
            &vertices[idx.z as usize],
            &Point3::from(Vector3::zeros()),
          )
        })
        .sum::<f32>()
        .abs()
    };

    let density = mass / volume;
    commands.insert_one(
      entity,
      ColliderBuilder::trimesh(vertices, indices).density(density),
    );

    Some(())
  }

  pub fn aabb(&self) -> AABB<f32> {
    let vertices = self.vertices();
    let indices = self.indices();
    info!("{} {}", vertices.len(), indices.len());
    let trimesh = NSTriMesh::new(
      vertices,
      indices.into_iter().map(|p| p.map(|n| n as usize)).collect(),
      None,
    );
    trimesh.aabb().clone()
  }

  pub fn build_collider(
    &self,
    commands: &mut Commands,
    entity: Entity,
    body_status: BodyStatus,
    mass: f32,
    debug_cube: Option<Handle<Mesh>>,
  ) -> Option<()> {
    match body_status {
      BodyStatus::Dynamic => self.build_approx_collider(commands, entity, mass, debug_cube),
      BodyStatus::Static => self.build_exact_collider(commands, entity, mass),
      BodyStatus::Kinematic => unimplemented!(),
    }
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

  fn vertices(&self) -> Vec<Point<f32>> {
    self
      .get_attribute(self.position_attribute.clone())
      .into_iter()
      .map(|point| Point3::from(point.coords.component_mul(&self.scale)))
      .collect()
  }

  fn indices(&self) -> Vec<Point<u32>> {
    match self.mesh.indices().as_ref().unwrap() {
      Indices::U32(indices) => indices
        .chunks(3)
        .map(|c| Point::new(c[0], c[1], c[2]))
        .collect::<Vec<_>>(),
      _ => unimplemented!(),
    }
  }

  fn to_ncollide_trimesh(&self) -> NTrimesh<f32> {
    let normals = self.get_attribute(self.normal_attribute.clone());

    NTrimesh::new(
      self.vertices(),
      Some(
        normals
          .clone()
          .into_iter()
          .map(|p| Matrix3x1::from_iterator(p.iter().cloned()))
          .collect(),
      ),
      None,
      Some(IndexBuffer::Unified(self.indices())),
    )
  }
}

// Sad hack because rapier types don't implement Reflect
#[allow(dead_code)]
#[derive(Reflect, Copy, Clone, PartialEq, Debug)]
#[reflect_value(PartialEq)]
pub enum AltBodyStatus {
  Static,
  Dynamic,
  Kinematic,
}

impl Default for AltBodyStatus {
  fn default() -> Self {
    AltBodyStatus::Dynamic
  }
}

impl AltBodyStatus {
  pub fn to_rapier(&self) -> BodyStatus {
    match self {
      AltBodyStatus::Static => BodyStatus::Static,
      AltBodyStatus::Dynamic => BodyStatus::Dynamic,
      AltBodyStatus::Kinematic => BodyStatus::Kinematic,
    }
  }
}

#[derive(Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ColliderParams {
  pub body_status: AltBodyStatus,
  pub mass: f32,
}

fn attach_collider(
  commands: &mut Commands,
  query: Query<(Entity, &ColliderParams, &Transform)>,
  children_query: Query<&Children>,
  mesh_query: Query<&Handle<Mesh>>,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  for (entity, collider_params, transform) in query.iter() {
    let _debug_cube = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));

    let body_status = collider_params.body_status.to_rapier();
    let (position, scale) = transform.to_na_isometry();

    if let Ok(mesh_handle) = mesh_query.get(entity) {
      let mesh = meshes.get(mesh_handle).unwrap();
      MeshWrapper::new(mesh, "Vertex_Position", "Vertex_Normal", scale)
        .build_collider(
          commands,
          entity,
          body_status,
          collider_params.mass,
          None, //Some(debug_cube),
        )
        .unwrap();
    } else {
      let children = utils::collect_children(entity, &children_query);

      // HACK: while scene is spawning, ignore this entity
      // is there a better way to listen for this?
      if children.len() == 0 {
        continue;
      }

      for child in children.iter() {
        info!("child {:?}", child);
        if let Ok(mesh_handle) = mesh_query.get(*child) {
          let mesh = meshes.get(mesh_handle).unwrap();
          MeshWrapper::new(mesh, "Vertex_Position", "Vertex_Normal", scale)
            .build_collider(
              commands,
              entity,
              body_status,
              collider_params.mass,
              None, //Some(_debug_cube.clone()),
            )
            .unwrap();
        }
      }
    }

    let rigid_body = RigidBodyBuilder::new(body_status)
      .position(position)
      .entity(entity);

    commands.set_current_entity(entity);
    commands.with(rigid_body);

    info!(
      "attached to {:?} with params {:?} with pos {:?}",
      entity, collider_params, position.translation
    );

    commands.remove_one::<ColliderParams>(entity);
  }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app.add_system(attach_collider.system());
    //app.add_startup_system(init_physics.system());
    //app.add_system(player_system.system());
  }
}

pub trait RigidBodyExt {
  fn entity(&self) -> Entity;
}

impl RigidBodyExt for RigidBody {
  fn entity(&self) -> Entity {
    Entity::from_bits(self.user_data as u64)
  }
}

pub trait RigidBodyBuilderExt {
  fn entity(self, entity: Entity) -> Self;
}

impl RigidBodyBuilderExt for RigidBodyBuilder {
  fn entity(self, entity: Entity) -> Self {
    self.user_data(entity.to_bits() as u128)
  }
}

pub trait AABBExt {
  fn volume(&self) -> f32;
}

impl AABBExt for AABB<f32> {
  fn volume(&self) -> f32 {
    let half_extents = self.half_extents();
    half_extents.x * half_extents.y * half_extents.z * 8.0
  }
}
