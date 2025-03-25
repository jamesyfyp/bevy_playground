use avian3d::{math::*, prelude::*};
use bevy::prelude::*;

use crate::GameState;
use crate::systems::camera::{camera_follow, pan_orbit_camera, spawn_camera};
use crate::systems::controller::{CharacterControllerBundle, PlayerMovementPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(PlayerMovementPlugin)
      .add_systems(OnEnter(GameState::Game), (setup, spawn_camera))
      .add_systems(
        Update,
        (camera_follow.before(pan_orbit_camera), pan_orbit_camera),
      )
      .add_systems(OnExit(GameState::Game), cleanup_game);
  }
}

#[derive(Component)]
pub struct InGameEntity;

#[derive(Component, Reflect)]
pub struct Player;

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  asset_server: Res<AssetServer>,
) {
  let grass_material = materials.add(StandardMaterial {
    base_color_texture: Some(asset_server.load("textures/grass.png")),
    ..default()
  });
  // circular base
  commands.spawn((
    RigidBody::Static,
    Collider::cylinder(20.0, 0.1),
    Mesh3d(meshes.add(Cylinder::new(20.0, 0.1))),
    MeshMaterial3d(grass_material),
    Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
    InGameEntity,
  ));
  // player
  commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
    Transform::from_xyz(0.0, 0.55, 0.0),
    Player,
    InGameEntity,
    CharacterControllerBundle::new(Collider::cuboid(1.0, 1.0, 1.0), Vector::NEG_Y * 5.81 * 2.0)
      .with_movement(30.0, 0.92, 7.0, (30.0 as Scalar).to_radians()),
  ));

  //balls for fun
  commands.spawn((
    RigidBody::Dynamic,
    Collider::sphere(0.2),
    ColliderDensity(3.0),
    SpeculativeMargin(5.0),
    Mesh3d(meshes.add(Sphere::new(0.2))),
    MeshMaterial3d(materials.add(Color::srgb_u8(255, 0, 0))),
    Transform::from_xyz(0.0, 1.0, 0.0),
    InGameEntity,
  ));
  // light
  commands.spawn((
    PointLight {
      shadows_enabled: true,
      ..default()
    },
    Transform::from_xyz(4.0, 8.0, 4.0),
    InGameEntity,
  ));
  // and all entries provided by the crate:
}

fn cleanup_game(mut commands: Commands, query: Query<Entity, With<InGameEntity>>) {
  for entity in query.iter() {
    commands.entity(entity).despawn_recursive();
  }
}
