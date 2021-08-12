use super::{controller, look};
use crate::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy_rapier3d::{prelude::*, rapier::geometry::InteractionGroups};

pub struct Player {
  pub body: Entity,
  pub head: Entity,
  pub camera: Entity,
}

pub const RAPIER_PLAYER_GROUP: u32 = 1;

pub fn spawn_character(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
  let height = 3.0;
  let head_scale = 0.3;

  let rigid_body = RigidBodyBundle {
    body_type: BodyStatus::Dynamic,
    position: vector![0., height * 3., 5.5].into(),
    mass_properties: RigidBodyMassProps {
      flags: RigidBodyMassPropsFlags::ROTATION_LOCKED,
      ..Default::default()
    },
    ..Default::default()
  };

  let collider = ColliderBundle {
    shape: ColliderShape::cuboid(1.0, height / 2., 1.0),
    mass_properties: ColliderMassProps::Density(1.0),
    flags: ColliderFlags {
      collision_groups: InteractionGroups::all().with_memberships(RAPIER_PLAYER_GROUP),
      ..Default::default()
    },
    ..Default::default()
  };

  let mut body = commands
    .spawn_bundle((
      controller::BodyTag,
      Transform::identity(),
      GlobalTransform::identity(),
      Name::new("player body"),
    ))
    .insert_bundle(rigid_body)
    .insert_bundle(collider)
    .insert(ColliderPositionSync::Discrete)
    .insert(ColliderDebugRender::with_id(1))
    .id();

  let cube = meshes.add(Mesh::from(bevy::prelude::shape::Cube { size: 2.0 }));
  let body_model = commands
    .spawn_bundle(PbrBundle {
      mesh: cube,
      transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
        Vec3::new(1.0, height / 2., 1.0),
        Quat::IDENTITY,
        Vec3::ZERO,
      )),
      ..Default::default()
    })
    .insert(Name::new("player body model"))
    .id();

  let yaw = commands
    .spawn_bundle((
      controller::YawTag,
      Transform::identity(),
      GlobalTransform::identity(),
      Name::new("player yaw"),
    ))
    .id();

  let head = commands
    .spawn_bundle((
      controller::HeadTag,
      GlobalTransform::identity(),
      Transform::from_matrix(Mat4::from_scale_rotation_translation(
        Vec3::ONE,
        Quat::from_rotation_y(0.),
        Vec3::new(0.0, 0.5 * head_scale + height - 1.695, 0.0),
      )),
      Name::new("player head"),
    ))
    .id();

  let perspective = controller::Perspective::FirstPerson;
  let camera = commands
    .spawn_bundle(PerspectiveCameraBundle {
      transform: perspective.to_transform(),
      //.looking_at(Vec3::new(0., height, 0.), Vec3::Y),
      ..Default::default()
    })
    .insert_bundle((
      look::LookDirection::default(),
      controller::CameraTag,
      perspective,
      Name::new("camera 3d"),
      //bevy_skybox::SkyboxCamera
    ))
    .id();

  commands
    .entity(body)
    .insert(look::LookEntity(camera))
    .push_children(&[yaw]);
  commands.entity(yaw).push_children(&[body_model, head]);
  commands.entity(head).push_children(&[camera]);

  commands.insert_resource(Player { body, head, camera });
}

pub fn init_hud(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  commands
    .spawn_bundle(UiCameraBundle::default())
    .insert(Name::new("camera ui"));

  commands
    .spawn_bundle(NodeBundle {
      style: Style {
        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
        position_type: PositionType::Absolute,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..Default::default()
      },
      material: materials.add(Color::NONE.into()),
      ..Default::default()
    })
    .insert(Name::new("crosshairs"))
    .with_children(|parent| {
      // bevy logo (image)
      parent.spawn_bundle(ImageBundle {
        style: Style {
          size: Size::new(Val::Px(30.0), Val::Auto),
          ..Default::default()
        },
        material: materials.add(asset_server.load("images/crosshairs.png").into()),
        ..Default::default()
      });
    });
}

// pub fn update_camera(
//   windows: Res<Windows>,
//   player: Res<Player>,
//   mut query: Query<&mut PerspectiveProjection>
// ) {
//   let window = windows.get_primary().unwrap();
//   let mut projection = query.get_mut(player.camera).unwrap();
//   projection.aspect_ratio = window.width() as f32 / window.height() as f32;
// }
