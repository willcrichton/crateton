use crate::prelude::*;

pub fn collect_children(parent: Entity, query: &Query<&Children>) -> Vec<Entity> {
  if let Ok(children) = query.get(parent) {
    children
      .iter()
      .map(|child| {
        let mut entities = collect_children(*child, query);
        entities.push(*child);
        entities
      })
      .flatten()
      .collect()
  } else {
    vec![]
  }
}
