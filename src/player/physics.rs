use super::{controller::*, events::*};

use bevy::prelude::*;
use bevy_rapier3d::{
  na::Vector3,
  physics::RigidBodyHandleComponent,
  rapier::{
    dynamics::{BodyStatus, RigidBodySet},
    math::Vector,
  },
};

pub fn create_mass(
  commands: &mut Commands,
  bodies: Res<RigidBodySet>,
  query: Query<(Entity, &RigidBodyHandleComponent), Without<Mass>>,
) {
  for (entity, body_handle) in &mut query.iter() {
    let body = bodies
      .get(body_handle.handle())
      .expect("Failed to get RigidBody");
    let mass = 1.0 / body.mass_properties().inv_mass;
    commands.insert_one(entity, Mass::new(mass));
  }
}

pub fn body_to_velocity(
  bodies: Res<RigidBodySet>,
  mut query: Query<&RigidBodyHandleComponent, With<BodyTag>>,
  mut controller: ResMut<CharacterController>,
) {
  for body_handle in query.iter_mut() {
    let body = bodies
      .get(body_handle.handle())
      .expect("Failed to get RigidBody");
    let velocity = body.linvel();
    controller.velocity = Vec3::new(velocity[0], velocity[1], velocity[2]);
  }
}

pub fn controller_to_rapier_dynamic_force(
  forces: Res<Events<ForceEvent>>,
  mut reader: ResMut<ControllerEvents>,
  mut bodies: ResMut<RigidBodySet>,
  mut query: Query<&RigidBodyHandleComponent, With<BodyTag>>,
  controller: ResMut<CharacterController>,
) {
  let mut force = Vec3::zero();
  for event in reader.forces.iter(&forces) {
    force += **event;
  }

  if force.length_squared() > 1E-6 {
    for body_handle in query.iter_mut() {
      if !controller.fly {
        let body = bodies
          .get_mut(body_handle.handle())
          .expect("Failed to get character body");
        body.body_status = BodyStatus::Dynamic;
        body.apply_force(Vector::new(force.x, force.y, force.z), true);
      }
    }
  }
}

pub fn controller_to_fly(
  translations: Res<Events<TranslationEvent>>,
  mut reader: ResMut<ControllerEvents>,
  mut bodies: ResMut<RigidBodySet>,
  mut query: Query<(&RigidBodyHandleComponent, &CharacterController), With<BodyTag>>,
) {
  for (body_handle, controller) in query.iter_mut() {
    if controller.fly {
      let body = bodies
        .get_mut(body_handle.handle())
        .expect("Failed to get character body");
      body.body_status = BodyStatus::Static;
      body.sleep();

      let mut position = body.position().clone();
      let delta = reader
        .translations
        .iter(&translations)
        .fold(Vec3::zero(), |a, b| a + **b);
      position.translation.vector += Vector3::new(delta.x, delta.y, delta.z);
      body.set_position(position, false);
    }
  }
}
