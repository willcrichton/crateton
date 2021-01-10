use super::{controller, look};

use bevy::prelude::*;
use bevy_rapier3d::{
  na::Vector3,
  rapier::{
    dynamics::RigidBodyBuilder,
    geometry::{ColliderBuilder, InteractionGroups},
  },
};

pub struct Player {
  pub body: Entity,
  pub head: Entity,
  pub camera: Entity,
}

pub const RAPIER_PLAYER_GROUP: u16 = 1;

pub fn spawn_character(commands: &mut Commands, mut meshes: ResMut<Assets<Mesh>>) {
  let height = 3.0;
  let head_scale = 0.3;
  let body = commands
    .spawn((
      controller::BodyTag,
      controller::CharacterController::default(),
      RigidBodyBuilder::new_dynamic()
        .translation(3., height * 2., 0.)
        .principal_angular_inertia(Vector3::zeros(), Vector3::repeat(false)),
      //ColliderBuilder::cylinder(0.5*height, 1.0).density(200.),
      ColliderBuilder::cuboid(1.0, 0.5 * height, 1.0)
        .collision_groups(InteractionGroups::all().with_groups(RAPIER_PLAYER_GROUP))
        .density(1.0),
      Transform::identity(),
      GlobalTransform::identity(),
    ))
    .current_entity()
    .unwrap();

  let cube = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));
  let body_model = commands
    .spawn(PbrBundle {
      mesh: cube,
      transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
        Vec3::new(1.0, 0.5 * height, 1.0),
        Quat::identity(),
        Vec3::zero(),
      )),
      ..Default::default()
    })
    .current_entity()
    .unwrap();

  let yaw = commands
    .spawn((
      controller::YawTag,
      Transform::identity(),
      GlobalTransform::identity(),
    ))
    .current_entity()
    .unwrap();

  let head = commands
    .spawn((
      controller::HeadTag,
      GlobalTransform::identity(),
      Transform::from_matrix(Mat4::from_scale_rotation_translation(
        Vec3::one(),
        Quat::from_rotation_y(0.),
        Vec3::new(0.0, 0.5 * head_scale + height - 1.695, 0.0),
      )),
    ))
    .current_entity()
    .unwrap();

  let perspective = controller::Perspective::FirstPerson;
  let camera = commands
    .spawn(Camera3dBundle {
      transform: perspective.to_transform(),
      ..Default::default()
    })
    .with_bundle((
      look::LookDirection::default(),
      controller::CameraTag,
      perspective,
    ))
    .current_entity()
    .unwrap();

  commands
    .insert_one(body, look::LookEntity(camera))
    .push_children(body, &[yaw])
    .push_children(yaw, &[body_model, head])
    .push_children(head, &[camera]);

  commands.insert_resource(Player { body, head, camera });
}

pub fn init_hud(
  commands: &mut Commands,
  asset_server: Res<AssetServer>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  commands
    .spawn(CameraUiBundle::default())
    .spawn(NodeBundle {
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
    .with_children(|parent| {
      // bevy logo (image)
      parent.spawn(ImageBundle {
        style: Style {
          size: Size::new(Val::Px(30.0), Val::Auto),
          ..Default::default()
        },
        material: materials.add(asset_server.load("images/crosshairs.png").into()),
        ..Default::default()
      });
    });
}
