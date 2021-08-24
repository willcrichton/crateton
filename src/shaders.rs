use bevy::{
  prelude::*,
  render::pipeline::{PipelineDescriptor, PipelineSpecialization, RenderPipeline},
  utils::HashSet,
};

use crate::physics::ColliderChildren;

pub struct AttachShaderEvent {
  pub entity: Entity,
  pub pipeline: Handle<PipelineDescriptor>,
}

pub struct DetachShaderEvent {
  pub entity: Entity,
  pub pipeline: Handle<PipelineDescriptor>,
}

fn handle_shader_events(
  mut attach_events: EventReader<AttachShaderEvent>,
  mut detach_events: EventReader<DetachShaderEvent>,
  mut render_pipelines_query: Query<&mut RenderPipelines>,
  mesh_query: Query<&Handle<Mesh>>,
  children_query: Query<&ColliderChildren>,
  meshes: Res<Assets<Mesh>>,
) {
  for AttachShaderEvent { entity, pipeline } in attach_events.iter() {
    let mut attach = |entity: Entity| {
      let mut render_pipelines = render_pipelines_query.get_mut(entity).unwrap();
      let specialization = render_pipelines.pipelines[0].specialization.clone();
      let render_pipeline = RenderPipeline::specialized(pipeline.clone(), specialization);
      render_pipelines.pipelines.push(render_pipeline);
    };

    if mesh_query.get(*entity).is_ok() {
      attach(*entity);
    } else {
      for child in children_query.get(*entity).unwrap().0.iter() {
        if mesh_query.get(*child).is_ok() {
          attach(*child);
        }
      }
    }
  }

  for DetachShaderEvent { entity, pipeline } in detach_events.iter() {
    // Remove shader from the set of render pipelines
    let mut detach = |entity: Entity| {
      let mut render_pipelines = render_pipelines_query.get_mut(entity).unwrap();
      render_pipelines.pipelines = render_pipelines
        .pipelines
        .clone()
        .into_iter()
        .filter(|p| p.pipeline != *pipeline)
        .collect();
    };

    if mesh_query.get(*entity).is_ok() {
      detach(*entity);
    } else {
      for child in children_query.get(*entity).unwrap().0.iter() {
        if mesh_query.get(*child).is_ok() {
          detach(*child);
        }
      }
    }
  }
}

pub struct ShadersPlugin;
impl Plugin for ShadersPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<AttachShaderEvent>()
      .add_event::<DetachShaderEvent>()
      .add_system(handle_shader_events.system());
  }
}
