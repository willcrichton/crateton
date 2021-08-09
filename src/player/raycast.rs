use super::{
  look::LookDirection,
  spawn::{Player, RAPIER_PLAYER_GROUP},
};
use crate::prelude::*;
use bevy_rapier3d::{
  na::{Point3, Vector3},
  prelude::*,
  rapier::{
    geometry::{ColliderHandle, InteractionGroups, Ray, RayIntersection},
    pipeline::QueryPipeline,
  },
};

#[derive(Debug)]
pub struct HitInfo {
  pub intersection: RayIntersection,
  pub entity: Entity,
}

pub struct ViewInfo {
  pub ray: Ray,
  pub hit: Option<HitInfo>,
}

impl Default for ViewInfo {
  fn default() -> Self {
    ViewInfo {
      ray: Ray::new(Point3::origin(), Vector3::zeros()),
      hit: None,
    }
  }
}

impl ViewInfo {
  pub fn hit_point(&self) -> Option<Point3<f32>> {
    self
      .hit
      .as_ref()
      .map(|hit| self.ray.point_at(hit.intersection.toi))
  }
}

pub fn compute_view_info(
  player: Res<Player>,
  look_direction_query: Query<&LookDirection>,
  global_transform_query: Query<&GlobalTransform>,
  rapier_pipeline: Res<QueryPipeline>,
  collider_query: QueryPipelineColliderComponentsQuery,
  collider_parent_query: Query<&ColliderParent>,
  mut view_info: ResMut<ViewInfo>,
) {
  let colliders = QueryPipelineColliderComponentsSet(&collider_query);
  let look = look_direction_query
    .get_component::<LookDirection>(player.camera)
    .expect("Failed to get LookDirection from Entity");
  let head_pos = global_transform_query.get(player.head).unwrap();
  let origin = head_pos.translation;
  let direction = look.forward;
  view_info.ray = Ray::new(origin.to_na_point3(), direction.to_na_vector3());
  view_info.hit = rapier_pipeline
    .cast_ray_and_get_normal(
      &colliders,
      &view_info.ray,
      f32::MAX,
      true,
      InteractionGroups::all().with_filter(u32::MAX ^ RAPIER_PLAYER_GROUP),
      None,
    )
    .map(|(collider_handle, intersection)| {
      let collider_entity = collider_handle.entity();
      let entity = match collider_parent_query.get(collider_entity) {
        Ok(parent) => parent.handle.entity(),
        _ => collider_entity,
      };
      HitInfo {
        intersection,
        entity,
      }
    });
}
