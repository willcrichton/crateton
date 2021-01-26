use bevy::{
  prelude::*,
  render::pipeline::{PipelineDescriptor, PipelineSpecialization, RenderPipeline},
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
      let specialization = {
        let mesh_handle = mesh_query.get(entity).unwrap();
        let mesh = meshes.get(mesh_handle).unwrap();
        PipelineSpecialization {
          vertex_buffer_descriptor: mesh.get_vertex_buffer_descriptor(),
          ..Default::default()
        }
      };

      // Add the shader to the set of render pipelines
      let mut render_pipelines = render_pipelines_query.get_mut(entity).unwrap();
      render_pipelines.pipelines.push(RenderPipeline::specialized(
        pipeline.clone(),
        specialization,
      ));
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
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_event::<AttachShaderEvent>()
      .add_event::<DetachShaderEvent>()
      .add_system(handle_shader_events.system());
  }
}
