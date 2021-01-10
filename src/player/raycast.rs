use super::{
  look::LookDirection,
  spawn::{Player, RAPIER_PLAYER_GROUP},
};
use crate::math::*;
use bevy::{ecs::SystemParam, prelude::*};
use bevy_rapier3d::rapier::{
  geometry::{Collider, ColliderHandle, ColliderSet, InteractionGroups, Ray, RayIntersection},
  pipeline::QueryPipeline,
};

#[derive(SystemParam)]
pub struct LookDirDeps<'a> {
  look_direction_query: Query<'a, &'a LookDirection>,
  global_transform_query: Query<'a, &'a GlobalTransform>,
}

#[derive(SystemParam)]
pub struct CastFromEyeDeps<'a> {
  rapier_pipeline: Res<'a, QueryPipeline>,
  colliders: Res<'a, ColliderSet>,
  look_dir_deps: LookDirDeps<'a>,
}

pub struct HitInfo<'a> {
  pub ray: Ray,
  pub collider_handle: ColliderHandle,
  pub collider: &'a Collider,
  pub intersection: RayIntersection,
  pub entity: Entity,
}

impl Player {
  pub fn look_dir<'a>(&self, deps: &'a LookDirDeps<'a>) -> Ray {
    let LookDirDeps {
      look_direction_query,
      global_transform_query,
    } = deps;
    let look = look_direction_query
      .get_component::<LookDirection>(self.camera)
      .expect("Failed to get LookDirection from Entity");
    let head_pos = global_transform_query.get(self.head).unwrap();
    let origin = head_pos.translation;
    let direction = look.forward;
    Ray::new(origin.to_point3(), direction.to_vector3())
  }

  pub fn cast_from_eye<'a>(&self, deps: &'a CastFromEyeDeps<'a>) -> Option<HitInfo<'a>> {
    let CastFromEyeDeps {
      rapier_pipeline,
      colliders,
      look_dir_deps,
    } = deps;

    let ray = self.look_dir(look_dir_deps);
    rapier_pipeline
      .cast_ray(
        &colliders,
        &ray,
        f32::MAX,
        InteractionGroups::all().with_mask(u16::MAX ^ RAPIER_PLAYER_GROUP),
      )
      .map(|(collider_handle, collider, intersection)| {
        let entity = Entity::from_bits(collider.user_data as u64);
        HitInfo {
          ray,
          collider_handle,
          collider,
          intersection,
          entity,
        }
      })
  }
}
