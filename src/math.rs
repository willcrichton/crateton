use bevy::prelude::*;
use bevy_rapier3d::na::{Point3, Quaternion, Vector3, Unit, UnitQuaternion};
pub trait NalgebraVecExt {
  fn to_glam_vec3(&self) -> Vec3;
}

impl NalgebraVecExt for Vector3<f32> {
  fn to_glam_vec3(&self) -> Vec3 {
    Vec3::new(self.x, self.y, self.z)
  }
}
  
impl NalgebraVecExt for Point3<f32> {
  fn to_glam_vec3(&self) -> Vec3 {
    Vec3::new(self.x, self.y, self.z)
  }
}

//pub trait NalgebraQuatExt {
  //fn to_quat(&self) -> Quat
//}

pub trait GlamVecExt {
  fn to_na_vector3(&self) -> Vector3<f32>;
  fn to_na_point3(&self) -> Point3<f32>;
}

impl GlamVecExt for Vec3 {
  fn to_na_vector3(&self) -> Vector3<f32> {
    Vector3::new(self.x, self.y, self.z)
  }

  fn to_na_point3(&self) -> Point3<f32> {
    Point3::new(self.x, self.y, self.z)
  }
}

pub trait GlamQuatExt {
  fn to_na_quat(&self) -> Quaternion<f32>;
  fn to_na_unit_quat(&self) -> UnitQuaternion<f32>;
}

impl GlamQuatExt for Quat {
  fn to_na_quat(&self) -> Quaternion<f32> {
    Quaternion::new(self.x, self.y, self.z, self.w)
  }

  fn to_na_unit_quat(&self) -> UnitQuaternion<f32> {
    Unit::new_normalize(self.to_na_quat())
  }
}