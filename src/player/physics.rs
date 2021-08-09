use super::{controller::*, events::*};
use crate::prelude::*;

use bevy_rapier3d::{
  na::Vector3,
  physics::RigidBodyHandleComponent,
  prelude::*,
  rapier::{dynamics::BodyStatus, math::Vector},
};

pub fn body_to_velocity(
  mut query: Query<&RigidBodyVelocity, With<BodyTag>>,
  mut controller: ResMut<CharacterController>,
) {
  for velocity in query.iter_mut() {
    controller.velocity = velocity.linvel.into();
  }
}

pub fn controller_to_rapier_dynamic_impulse(
  mut impulses: EventReader<ImpulseEvent>,
  mut query: Query<
    (
      &RigidBodyMassProps,
      &mut RigidBodyVelocity,
      &mut RigidBodyType,
      &mut RigidBodyActivation,
    ),
    With<BodyTag>,
  >,
  controller: ResMut<CharacterController>,
) {
  let mut impulse = Vec3::zero();
  for event in impulses.iter() {
    impulse += **event;
  }

  if impulse.length_squared() > 1E-6 {
    for (mass_props, mut velocity, mut body_type, mut activation) in query.iter_mut() {
      if !controller.fly {
        *body_type = BodyStatus::Dynamic;
        velocity.apply_impulse(mass_props, impulse.into());
        activation.wake_up(true);
      }
    }
  }
}

pub fn controller_to_fly(
  mut translations: EventReader<TranslationEvent>,
  mut query: Query<
    (
      &mut RigidBodyPosition,
      &mut RigidBodyActivation,
      &mut RigidBodyType,
    ),
    With<BodyTag>,
  >,
  controller: Res<CharacterController>,
) {
  for (mut body_position, mut body_activation, mut body_type) in query.iter_mut() {
    if controller.fly {
      *body_type = BodyStatus::Static;
      body_activation.sleep();

      let delta = translations.iter().fold(Vec3::zero(), |a, b| a + **b);
      body_position.next_position = body_position.position;
      body_position.next_position.translation.vector += delta.to_na_vector3();
    }
  }
}
