use super::{
  look::LookDirection,
  spawn::{Player, RAPIER_PLAYER_GROUP},
};
use crate::prelude::*;
use bevy::{ecs::SystemParam, prelude::*};
use bevy_rapier3d::rapier::{
  dynamics::RigidBodySet,
  geometry::{ColliderHandle, ColliderSet, InteractionGroups, Ray, RayIntersection},
  pipeline::QueryPipeline,
};

#[derive(SystemParam)]
pub struct LookDirDeps<'a> {
  pub look_direction_query: Query<'a, &'a LookDirection>,
  pub global_transform_query: Query<'a, &'a GlobalTransform>,
}

#[derive(SystemParam)]
pub struct CastFromEyeDeps<'a> {
  pub rapier_pipeline: Res<'a, QueryPipeline>,
  pub colliders: Res<'a, ColliderSet>,
  pub look_dir_deps: LookDirDeps<'a>,
  pub bodies: ResMut<'a, RigidBodySet>,
}

pub struct HitInfo {
  pub ray: Ray,
  pub collider_handle: ColliderHandle,
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
    Ray::new(origin.to_na_point3(), direction.to_na_vector3())
  }

  pub fn cast_from_eye<'a>(&self, deps: &CastFromEyeDeps<'a>) -> Option<HitInfo> {
    let CastFromEyeDeps {
      rapier_pipeline,
      colliders,
      look_dir_deps,
      bodies,
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
        let entity = bodies.get(collider.parent()).unwrap().entity();
        HitInfo {
          ray,
          collider_handle,
          intersection,
          entity,
        }
      })
  }
}
