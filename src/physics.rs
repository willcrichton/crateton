use crate::{
  models::{mesh_wrapper::MeshWrapper, ModelInstance},
  prelude::*,
  utils,
};
use bevy::{
  gltf::GltfId,
  reflect::{self as bevy_reflect, TypeUuid},
  render::mesh::{Indices, VertexAttributeValues},
};
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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const POSITION_ATTRIBUTE: &'static str = "Vertex_Position";
pub const NORMAL_ATTRIBUTE: &'static str = "Vertex_Normal";

pub type MeshComponent = (Isometry<f32>, Vec<Point<f32>>, Vec<[u32; 3]>);

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "ae35a361-fb0b-47a1-97f7-c9e68091eec4"]
pub struct SceneDecomposition(pub HashMap<GltfId, Vec<MeshComponent>>);

pub fn build_collider(
  mut commands: EntityCommands,
  mesh: &Mesh,
  scale: &Vector3<f32>,
  body_status: BodyStatus,
  decomp: Option<&Vec<MeshComponent>>,
) {
  let scale_vertices = |vs: &mut [Point<f32>]| {
    for v in vs.iter_mut() {
      *v = Point3::from(v.coords.component_mul(scale));
    }
  };

  let shape = match (body_status, decomp) {
    (BodyStatus::Dynamic, Some(decomp)) => {
      let compound = decomp
        .iter()
        .map(|(offset, vertices, _indices)| {
          let mut vertices = vertices.clone();
          scale_vertices(&mut vertices);
          (
            offset.clone(),
            ColliderShape::convex_hull(&vertices).unwrap(),
          )
        })
        .collect::<Vec<_>>();
      ColliderShape::compound(compound)
    }
    _ => {
      let mesh_wrapper = MeshWrapper::new(mesh, POSITION_ATTRIBUTE, NORMAL_ATTRIBUTE);
      let mut vertices = mesh_wrapper.vertices();
      scale_vertices(&mut vertices);
      let indices = mesh_wrapper.indices();
      if body_status == BodyStatus::Dynamic {
        ColliderShape::convex_decomposition(&vertices, &indices)
      } else {
        ColliderShape::trimesh(vertices, indices)
      }
    }
  };

  let id = commands.id().id() as usize;
  commands.insert_bundle(ColliderBundle {
    shape,
    mass_properties: ColliderMassProps::Density(1.0),
    ..Default::default()
  });
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
  gltf_id_query: Query<&GltfId>,
  decomp_query: Query<&SceneDecomposition>,
  mesh_query: Query<&Handle<Mesh>>,
  transform_query: Query<&GlobalTransform>,
  mut meshes: ResMut<Assets<Mesh>>,
  scene_spawner: Res<SceneSpawner>,
) {
  for (entity, model_instance, collider_params) in query.iter_mut() {
    let body_status = collider_params.body_status;
    let (global_position, global_scale) = transform_query.get(entity).unwrap().to_na_isometry();

    if let Ok(mesh_handle) = mesh_query.get(entity) {
      info!("Attaching collider directly to entity: {:?}", entity);
      let mesh = meshes.get(mesh_handle).unwrap();
      build_collider(
        commands.entity(entity),
        mesh,
        &global_scale,
        body_status,
        None,
      );
    } else {
      let children = utils::collect_children(entity, &children_query);
      let decomp = model_instance.and_then(|model| decomp_query.get(model.0).ok());

      // HACK: while scene is spawning, ignore this entity
      // is there a better way to listen for this?
      if children.len() == 0 || (body_status == BodyStatus::Dynamic && decomp.is_none()) {
        continue;
      }

      info!("Attaching collider to children of entity: {:?}", entity);

      for child in children.iter() {
        if let Ok(mesh_handle) = mesh_query.get(*child) {
          let mesh = meshes.get(mesh_handle).unwrap();
          let (child_position, child_scale) = transform_query.get(*child).unwrap().to_na_isometry();
          let compound =
            decomp.map(|decomp| decomp.0.get(gltf_id_query.get(*child).unwrap()).unwrap());
          build_collider(
            commands.entity(*child),
            mesh,
            &child_scale,
            body_status,
            compound,
          );

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
