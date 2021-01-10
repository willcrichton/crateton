use bevy::prelude::*;
use bevy_rapier3d::na::{Point3, Vector3};
pub trait NalgebraExt {
  fn to_vec3(&self) -> Vec3;
}

impl NalgebraExt for Vector3<f32> {
  fn to_vec3(&self) -> Vec3 {
    Vec3::new(self.x, self.y, self.z)
  }
}
  
impl NalgebraExt for Point3<f32> {
  fn to_vec3(&self) -> Vec3 {
    Vec3::new(self.x, self.y, self.z)
  }
}

pub trait GlamExt {
  fn to_vector3(&self) -> Vector3<f32>;
  fn to_point3(&self) -> Point3<f32>;
}

impl GlamExt for Vec3 {
  fn to_vector3(&self) -> Vector3<f32> {
    Vector3::new(self.x, self.y, self.z)
  }

  fn to_point3(&self) -> Point3<f32> {
    Point3::new(self.x, self.y, self.z)
  }
}