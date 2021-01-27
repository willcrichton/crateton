use crate::{
  models::{MeshDecomposition, ModelInstance, SceneDecomposition},
  prelude::*,
  utils,
};
use bevy::{
  prelude::*,
  render::mesh::{Indices, VertexAttributeValues},
  scene::LatestInstance,
};
use bevy_rapier3d::{
  na::{Matrix3x1, Point3, Vector3},
  rapier::{
    dynamics::{BodyStatus, IntegrationParameters, RigidBody, RigidBodyBuilder},
    geometry::ColliderBuilder,
    math::Point,
  },
};
use ncollide3d::{
  bounding_volume::{HasBoundingVolume, AABB},
  procedural::{IndexBuffer, TriMesh as NTrimesh},
  shape::TriMesh as NSTriMesh,
};
use std::borrow::Cow;

pub struct MeshWrapper<'a> {
  mesh: &'a Mesh,
  normal_attribute: String,
  position_attribute: String,
  scale: Vector3<f32>,
}

const HACD_ERROR: f32 = 0.10;
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

  pub fn compute_decomposition(&self, decomp: Option<&MeshDecomposition>) -> Vec<NTrimesh<f32>> {
    decomp
      .map(|decomp| decomp.to_trimesh(&self.scale))
      .unwrap_or_else(|| {
        let trimesh = self.to_ncollide_trimesh();
        let (decomp, _partition) =
          ncollide3d::transformation::hacd(trimesh, HACD_ERROR, HACD_MIN_COMPONENTS);

        decomp
      })
  }

  fn build_approx_collider(
    &self,
    commands: &mut Commands,
    entity: Entity,
    debug_cube: Option<Handle<Mesh>>,
    decomp: Option<&MeshDecomposition>,
  ) -> Option<()> {
    let colliders = self
      .compute_decomposition(decomp)
      .into_iter()
      .map(|trimesh| {
        // Get AABB from trimesh
        let aabb: AABB<_> = NSTriMesh::from(trimesh).local_bounding_volume();

        // If HACD computes an AABB with zero extent in any dimension, collider will end up with zero volume
        // and hence zero mass. So we add a small epsilon
        let half_extents = aabb.half_extents().add_scalar(HACD_EPSILON);
        let aabb = AABB::from_half_extents(aabb.center(), half_extents);
        let center = aabb.center();

        let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
          .translation(center.x, center.y, center.z);

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

  fn build_exact_collider(&self, commands: &mut Commands, entity: Entity) -> Option<()> {
    let vertices = self.vertices();
    let indices = self.indices();
    commands.insert_one(entity, ColliderBuilder::trimesh(vertices, indices));

    Some(())
  }

  pub fn build_collider(
    &self,
    commands: &mut Commands,
    entity: Entity,
    body_status: BodyStatus,
    debug_cube: Option<Handle<Mesh>>,
    decomp: Option<&MeshDecomposition>,
  ) -> Option<()> {
    match body_status {
      BodyStatus::Dynamic => self.build_approx_collider(commands, entity, debug_cube, decomp),
      BodyStatus::Static => self.build_exact_collider(commands, entity),
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

#[derive(Debug)]
pub struct ColliderParams {
  pub body_status: BodyStatus,
  pub mass: f32,
}

pub struct ColliderChildren(pub Vec<Entity>);

fn attach_collider(
  commands: &mut Commands,
  mut query: Query<(Entity, Option<&ModelInstance>, &ColliderParams)>,
  children_query: Query<&Children>,
  mesh_query: Query<&Handle<Mesh>>,
  decomp_query: Query<&SceneDecomposition>,
  instance_query: Query<&LatestInstance>,
  mut transform_query: Query<&mut Transform>,
  mut meshes: ResMut<Assets<Mesh>>,
  scene_spawner: Res<SceneSpawner>,
) {
  for (entity, model_instance, collider_params) in query.iter_mut() {
    let _debug_cube = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));

    let body_status = collider_params.body_status;
    let (position, scale) = transform_query.get_mut(entity).unwrap().to_na_isometry();

    if let Ok(mesh_handle) = mesh_query.get(entity) {
      let mesh = meshes.get(mesh_handle).unwrap();
      MeshWrapper::new(mesh, "Vertex_Position", "Vertex_Normal", scale)
        .build_collider(
          commands,
          entity,
          body_status,
          None, //Some(debug_cube),
          None,
        )
        .unwrap();
    } else {
      let children = utils::collect_children(entity, &children_query);

      // HACK: while scene is spawning, ignore this entity
      // is there a better way to listen for this?
      if children.len() == 0 {
        continue;
      }

      let model = &model_instance.unwrap().0;
      let decomp = decomp_query.get(*model).ok();
      let instance = &instance_query.get(entity).unwrap().0;
      let entity_map = scene_spawner.entity_map(instance.clone()).unwrap();

      transform_query.get_mut(entity).unwrap().scale = Vec3::new(1., 1., 1.);

      for child in children.iter() {
        if let Ok(mesh_handle) = mesh_query.get(*child) {
          let mesh = meshes.get(mesh_handle).unwrap();
          transform_query.get_mut(*child).unwrap().scale = scale.to_glam_vec3();
          MeshWrapper::new(mesh, "Vertex_Position", "Vertex_Normal", scale)
            .build_collider(
              commands,
              entity,
              body_status,
              None, //Some(_debug_cube.clone()),
              decomp.map(|decomp| {
                let inst_entity = entity_map
                  .keys()
                  .find(|k| entity_map.get(*k).unwrap() == *child)
                  .unwrap();
                decomp.meshes.get(&inst_entity).unwrap()
              }),
            )
            .unwrap();
        } else {
          if let Ok(mut transform) = transform_query.get_mut(*child) {
            transform.scale = Vec3::new(1., 1., 1.);
          }
        }
      }

      commands.insert_one(entity, ColliderChildren(children));
    }

    let rigid_body = RigidBodyBuilder::new(body_status)
      .position(position)
      .entity(entity)
      .mass(collider_params.mass, false);

    commands.set_current_entity(entity);
    commands.with(rigid_body);

    commands.remove_one::<ColliderParams>(entity);
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

// TODO: configure rapier to avoid clipping issues
fn configure_rapier(mut integration_params: ResMut<IntegrationParameters>) {
  integration_params.ccd_on_penetration_enabled = true;
  integration_params.max_velocity_iterations *= 2;
  integration_params.max_position_iterations *= 2;
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_plugin(bevy_rapier3d::physics::RapierPhysicsPlugin)
      .add_system(attach_collider.system())
      .add_startup_system(configure_rapier.system());
  }
}
