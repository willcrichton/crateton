use crate::prelude::*;
use std::ops::Deref;

macro_rules! event_wrapper {
  ($name:ident, $inner:ty) => {
    pub struct $name(pub $inner);
    impl Deref for $name {
      type Target = $inner;
      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }
  };
}

event_wrapper!(LookDeltaEvent, Vec3);
event_wrapper!(LookEvent, Vec3);
event_wrapper!(PitchEvent, f32);
event_wrapper!(YawEvent, f32);
event_wrapper!(TranslationEvent, Vec3);
event_wrapper!(ImpulseEvent, Vec3);
event_wrapper!(ForceEvent, Vec3);
