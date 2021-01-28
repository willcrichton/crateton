use bevy::{
  app::AppExit,
  prelude::*,
  render::{
    render_graph::{Node, ResourceSlotInfo, ResourceSlots},
    renderer::{
      BufferId, BufferInfo, BufferMapMode, BufferUsage, RenderContext, RenderResourceId,
      RenderResourceType,
    },
    texture::TextureDescriptor,
  },
};
use std::{borrow::Cow, marker::Sync};

pub struct TextureNode {
  descriptor: TextureDescriptor,
}

impl TextureNode {
  pub const OUT_TEXTURE: &'static str = "texture";

  pub fn new(descriptor: TextureDescriptor) -> Self {
    Self { descriptor }
  }
}

impl Node for TextureNode {
  fn output(&self) -> &[ResourceSlotInfo] {
    static OUTPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
      name: Cow::Borrowed(TextureNode::OUT_TEXTURE),
      resource_type: RenderResourceType::Texture,
    }];
    OUTPUT
  }

  fn update(
    &mut self,
    _world: &World,
    _resources: &Resources,
    render_context: &mut dyn RenderContext,
    _input: &ResourceSlots,
    output: &mut ResourceSlots,
  ) {
    const TEXTURE: usize = 0;
    if output.get(TEXTURE).is_none() {
      let render_resource_context = render_context.resources_mut();
      let texture_resource = render_resource_context.create_texture(self.descriptor);
      output.set(TEXTURE, RenderResourceId::Texture(texture_resource));
    }
  }
}

pub struct TextureReadoutNode {
  descriptor: TextureDescriptor,
  texture_buffer: Option<BufferId>,
  texture_buffer_size: usize,
  read: Box<dyn Fn(&[u8], TextureDescriptor) + Send + Sync>,
  pub ready: bool,
}

impl TextureReadoutNode {
  pub const IN_TEXTURE: &'static str = "texture";

  pub fn new(
    descriptor: TextureDescriptor,
    read: impl Fn(&[u8], TextureDescriptor) + 'static + Send + Sync,
  ) -> Self {
    Self {
      descriptor,
      texture_buffer: None,
      texture_buffer_size: 0,
      read: Box::new(read),
      ready: false,
    }
  }
}

impl Node for TextureReadoutNode {
  fn input(&self) -> &[ResourceSlotInfo] {
    static INPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
      name: Cow::Borrowed(TextureReadoutNode::IN_TEXTURE),
      resource_type: RenderResourceType::Texture,
    }];
    INPUT
  }

  fn update(
    &mut self,
    _world: &World,
    resources: &Resources,
    render_context: &mut dyn RenderContext,
    input: &ResourceSlots,
    _output: &mut ResourceSlots,
  ) {
    if !self.ready {
      return;
    }

    let mut app_exit_events = resources.get_mut::<Events<AppExit>>().unwrap();

    if let Some(RenderResourceId::Texture(texture)) = input.get(0) {
      let render_resource_context = render_context.resources_mut();
      let width = self.descriptor.size.width as usize;
      let aligned_width = render_resource_context.get_aligned_texture_size(width);
      let format_size = self.descriptor.format.pixel_size();

      let texture_buffer = match self.texture_buffer {
        Some(buffer) => buffer,
        None => {
          let buffer_size = self.descriptor.size.volume() * format_size;
          let buffer = render_resource_context.create_buffer(BufferInfo {
            size: buffer_size,
            buffer_usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            mapped_at_creation: false,
          });
          self.texture_buffer = Some(buffer);
          self.texture_buffer_size = buffer_size;
          buffer
        }
      };

      render_context.copy_texture_to_buffer(
        texture,
        [0, 0, 0],
        0,
        texture_buffer,
        0,
        (format_size * aligned_width) as u32,
        self.descriptor.size,
      );

      let render_resource_context = render_context.resources_mut();
      let reader = &self.read;
      render_resource_context.map_buffer(texture_buffer, BufferMapMode::Read);
      render_resource_context.write_mapped_buffer(
        texture_buffer,
        0..self.texture_buffer_size as u64,
        &mut |data, _renderer| {
          // COMPLETE HACK: better way to wait for non-black images to render?
          if data[0] == 0 && data[1] == 0 && data[2] == 0 {
            return;
          }

          reader(data.as_ref(), self.descriptor);
          app_exit_events.send(AppExit);
        },
      );
      render_resource_context.unmap_buffer(texture_buffer);
    }
  }
}
