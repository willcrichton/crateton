use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::{
    camera::{ActiveCameras, Camera, CameraProjection},
    pass::{
      LoadOp, Operations, PassDescriptor, RenderPassColorAttachment,
      RenderPassDepthStencilAttachment, TextureAttachment,
    },
    render_graph::{
      base::{node::MAIN_PASS, MainPass},
      CameraNode, PassNode, RenderGraph, TextureNode,
    },
    texture::{
      Extent3d, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
    },
  },
  window::WindowId,
};
use std::path::PathBuf;

pub struct FirstPass;

// pub const RENDER_TEXTURE_HANDLE: HandleUntyped =
//   HandleUntyped::weak_from_u64(Texture::TYPE_UUID, 13378939762009864029);

const TEXTURE_NODE: &str = "texure_node";
const DEPTH_TEXTURE_NODE: &str = "depth_texure_node";
const FIRST_PASS: &str = "first_pass";
const FIRST_PASS_CAMERA: &str = "first_pass_camera";

#[derive(Default)]
pub struct RenderTextureHandle(Handle<Texture>);

pub fn add_render_to_texture_graph(
  commands: &mut Commands,
  graph: &mut RenderGraph,
  size: Extent3d,
  active_cameras: &mut ActiveCameras,
  mut textures: ResMut<Assets<Texture>>,
  mut render_texture_handle: ResMut<RenderTextureHandle>,
) {
  let mut first_pass_camera = PerspectiveCameraBundle {
    camera: Camera {
      name: Some(FIRST_PASS_CAMERA.to_string()),
      window: WindowId::new(), /* otherwise it will use main window size / aspect for
                                * calculation of projection matrix */
      ..Default::default()
    },
    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
      .looking_at(Vec3::default(), Vec3::Y),
    ..Default::default()
  };
  active_cameras.add(FIRST_PASS_CAMERA);

  let camera_projection = &mut first_pass_camera.perspective_projection;
  camera_projection.update(size.width as f32, size.height as f32);
  first_pass_camera.camera.projection_matrix = camera_projection.get_projection_matrix();
  first_pass_camera.camera.depth_calculation = camera_projection.depth_calculation();
  commands.spawn_bundle(first_pass_camera);

  let mut pass_node = PassNode::<&FirstPass>::new(PassDescriptor {
    color_attachments: vec![RenderPassColorAttachment {
      attachment: TextureAttachment::Input("color_attachment".to_string()),
      resolve_target: None,
      ops: Operations {
        load: LoadOp::Clear(Color::rgb(0.1, 0.2, 0.3)),
        store: true,
      },
    }],
    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
      attachment: TextureAttachment::Input("depth".to_string()),
      depth_ops: Some(Operations {
        load: LoadOp::Clear(1.0),
        store: true,
      }),
      stencil_ops: None,
    }),
    sample_count: 1,
  });

  pass_node.add_camera(FIRST_PASS_CAMERA);

  graph.add_node(FIRST_PASS, pass_node);
  graph.add_system_node(FIRST_PASS_CAMERA, CameraNode::new(FIRST_PASS_CAMERA));
  graph.add_node_edge(FIRST_PASS_CAMERA, FIRST_PASS).unwrap();

  render_texture_handle.0 = textures.add(Texture::new(
    size,
    TextureDimension::D2,
    vec![0; size.volume() * std::mem::size_of::<f32>()],
    Default::default(),
  ));

  graph.add_node(
    TEXTURE_NODE,
    TextureNode::new(
      TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: Default::default(),
        usage: TextureUsage::WRITE_ALL | TextureUsage::READ_ALL,
      },
      Some(SamplerDescriptor::default()),
      Some(render_texture_handle.0.clone_untyped()),
    ),
  );

  graph.add_node(
    DEPTH_TEXTURE_NODE,
    TextureNode::new(
      TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
      },
      None,
      None,
    ),
  );

  graph.add_node_edge(TEXTURE_NODE, FIRST_PASS).unwrap();
  graph
    .add_slot_edge(
      TEXTURE_NODE,
      TextureNode::TEXTURE,
      FIRST_PASS,
      "color_attachment",
    )
    .unwrap();
  graph
    .add_slot_edge(
      DEPTH_TEXTURE_NODE,
      TextureNode::TEXTURE,
      FIRST_PASS,
      "depth",
    )
    .unwrap();
  graph.add_node_edge(FIRST_PASS, MAIN_PASS).unwrap();
  graph.add_node_edge("transform", FIRST_PASS).unwrap();
}

pub fn texture_system(
  textures: Res<Assets<Texture>>,
  render_texture_handle: Res<RenderTextureHandle>,
) {
  if let Some(texture) = textures.get(&render_texture_handle.0) {
    println!("{:?}", &texture.data[..18]);
  }
}
