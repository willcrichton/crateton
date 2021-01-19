use bevy::{
  prelude::*,
  render::pipeline::{PipelineDescriptor, PipelineSpecialization, RenderPipeline},
};


struct AttachShaderEvent {
  entity: Entity,
  pipeline: Handle<PipelineDescriptor>,
}

struct DetachShaderEvent {
  entity: Entity,
  pipeline: Handle<PipelineDescriptor>,
}

#[derive(Default)]
pub struct ShaderEvents {
  attach: Events<AttachShaderEvent>,
  detach: Events<DetachShaderEvent>,
}

#[derive(Default)]
pub struct ShaderEventReaders {
  attach: EventReader<AttachShaderEvent>,
  detach: EventReader<DetachShaderEvent>,
}

impl ShaderEvents {
  pub fn attach_shader(&mut self, entity: Entity, pipeline: Handle<PipelineDescriptor>) {
    self.attach.send(AttachShaderEvent { entity, pipeline });
  }

  pub fn detach_shader(&mut self, entity: Entity, pipeline: Handle<PipelineDescriptor>) {
    self.detach.send(DetachShaderEvent { entity, pipeline });
  }
}

fn handle_shader_events(
  mut readers: ResMut<ShaderEventReaders>,
  events: Res<ShaderEvents>,
  mut render_pipelines_query: Query<&mut RenderPipelines>,
  mesh_query: Query<&Handle<Mesh>>,
  meshes: Res<Assets<Mesh>>,
) {
  for AttachShaderEvent { entity, pipeline } in readers.attach.iter(&events.attach) {
    // Get the entity's mesh so we can specialize the shader to its attributes
    let specialization = {
      let mesh_handle = mesh_query.get(*entity).unwrap();
      let mesh = meshes.get(mesh_handle).unwrap();
      PipelineSpecialization {
        vertex_buffer_descriptor: mesh.get_vertex_buffer_descriptor(),
        ..Default::default()
      }
    };

    // Add the shader to the set of render pipelines
    let mut render_pipelines = render_pipelines_query.get_mut(*entity).unwrap();
    render_pipelines.pipelines.push(RenderPipeline::specialized(
      pipeline.clone(),
      specialization,
    ));
  }

  for DetachShaderEvent { entity, pipeline } in readers.detach.iter(&events.detach) {
    // Remove shader from the set of render pipelines
    let mut render_pipelines = render_pipelines_query.get_mut(*entity).unwrap();
    render_pipelines.pipelines = render_pipelines
      .pipelines
      .clone()
      .into_iter()
      .filter(|p| p.pipeline != *pipeline)
      .collect();
  }
}

pub struct ShadersPlugin;
impl Plugin for ShadersPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<ShaderEvents>()
      .init_resource::<ShaderEventReaders>()
      .add_event::<AttachShaderEvent>()
      .add_event::<DetachShaderEvent>()
      .add_system(handle_shader_events.system());
  }
}
