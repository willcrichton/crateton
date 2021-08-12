use crate::{models::ModelInstance, prelude::*, utils};
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy_rapier3d::{
  na::{Point3, UnitQuaternion, Vector3},
  prelude::*,
  rapier::{
    dynamics::{BodyStatus, IntegrationParameters},
    math::Point,
    parry::{
      bounding_volume::AABB,
      shape::TriMesh,
      transformation::vhacd::{VHACDParameters, VHACD},
    },
  },
};

use std::borrow::Cow;

pub struct MeshWrapper<'a> {
  mesh: &'a Mesh,
  normal_attribute: String,
  position_attribute: String,
  scale: Vector3<f32>,
}

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

  pub fn build_collider(&self, mut commands: EntityCommands, body_status: BodyStatus) {
    let vertices = self.vertices();
    let indices = self.indices();
    let shape = if body_status == BodyStatus::Dynamic {
      ColliderShape::convex_decomposition(&vertices, &indices)
    } else {
      ColliderShape::trimesh(vertices, indices)
    };

    let id = commands.id().id() as usize;
    commands.insert_bundle(ColliderBundle {
      shape,
      mass_properties: ColliderMassProps::Density(1.0),
      ..Default::default()
    });
  }

  fn get_attribute(&self, name: impl Into<Cow<'static, str>>) -> Vec<Point<f32>> {
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

  fn vertices(&self) -> Vec<Point<f32>> {
    self
      .get_attribute(self.position_attribute.clone())
      .into_iter()
      .map(|point| Point3::from(point.coords.component_mul(&self.scale)))
      .collect()
  }

  fn indices(&self) -> Vec<[u32; 3]> {
    match self.mesh.indices().as_ref().unwrap() {
      Indices::U32(indices) => indices
        .chunks(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect::<Vec<_>>(),
      _ => unimplemented!(),
    }
  }
}

#[derive(Debug)]
pub struct ColliderParams {
  pub body_status: BodyStatus,
  pub mass: f32,
}

pub struct ColliderChildren(pub Vec<Entity>);

fn attach_collider(
  mut commands: Commands,
  mut query: Query<(Entity, Option<&ModelInstance>, &ColliderParams)>,
  children_query: Query<&Children>,
  mesh_query: Query<&Handle<Mesh>>,
  transform_query: Query<&GlobalTransform>,
  mut meshes: ResMut<Assets<Mesh>>,
  scene_spawner: Res<SceneSpawner>,
) {
  for (entity, model_instance, collider_params) in query.iter_mut() {
    let body_status = collider_params.body_status;
    let (global_position, global_scale) = transform_query.get(entity).unwrap().to_na_isometry();

    if let Ok(mesh_handle) = mesh_query.get(entity) {
      let mesh = meshes.get(mesh_handle).unwrap();
      MeshWrapper::new(mesh, "Vertex_Position", "Vertex_Normal", global_scale)
        .build_collider(commands.entity(entity), body_status);
    } else {
      let children = utils::collect_children(entity, &children_query);
      println!("SPAWN? {:?}", children.len());

      // HACK: while scene is spawning, ignore this entity
      // is there a better way to listen for this?
      if children.len() == 0 {
        continue;
      }

      for child in children.iter() {
        if let Ok(mesh_handle) = mesh_query.get(*child) {
          let mesh = meshes.get(mesh_handle).unwrap();
          let (child_position, child_scale) = transform_query.get(*child).unwrap().to_na_isometry();
          MeshWrapper::new(mesh, "Vertex_Position", "Vertex_Normal", child_scale)
            .build_collider(commands.entity(*child), body_status);

          let pos_wrt_parent = Isometry::from_parts(
            (child_position.translation.vector - global_position.translation.vector).into(),
            global_position
              .rotation
              .rotation_to(&child_position.rotation),
          );
          let handle = entity.handle();
          commands.entity(*child).insert(ColliderParent {
            handle,
            pos_wrt_parent,
          });
        }
      }

      commands.entity(entity).insert(ColliderChildren(children));
    }

    let mut mass_properties = RigidBodyMassProps::default();
    mass_properties.local_mprops.set_mass(1., true);

    let rigid_body = RigidBodyBundle {
      body_type: body_status,
      mass_properties,
      position: global_position.into(),
      ..Default::default()
    };

    commands
      .entity(entity)
      .insert_bundle(rigid_body)
      .insert(RigidBodyPositionSync::Discrete)
      .remove::<ColliderParams>();
  }
}

pub trait AABBExt {
  fn volume(&self) -> f32;
}

impl AABBExt for AABB {
  fn volume(&self) -> f32 {
    let half_extents = self.half_extents();
    half_extents.x * half_extents.y * half_extents.z * 8.0
  }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
      .add_system(attach_collider.system());
  }
}
