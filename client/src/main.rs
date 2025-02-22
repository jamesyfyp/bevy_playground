use bevy::prelude::*;
use bevy::window::{WindowPlugin, WindowTheme, PresentMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use avian3d::prelude::*;

mod systems;
use systems::camera::{spawn_camera, pan_orbit_camera, PanOrbitState};


fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "I am a window!".into(),
                name: Some("bevy.app".into()),
                //mode: WindowMode::Fullscreen(MonitorSelection::Primary),
                present_mode: PresentMode::AutoVsync,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }),
        WorldInspectorPlugin::new(),
        PhysicsPlugins::default(), 
    ))
    .add_systems(Startup, (setup, spawn_camera))
    .add_systems(Update, (player_movement.before(pan_orbit_camera), pan_orbit_camera))
    .run();
}

#[derive(Component)]
struct Player;

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
    ));
    // player
    commands.spawn((
        RigidBody::Kinematic,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(3.0),
        SpeculativeMargin(5.0),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.55, 0.0),
        Player,
    ));

    //balls for fun
    commands.spawn(
        (RigidBody::Dynamic,
        Collider::sphere(0.2),
        ColliderDensity(3.0),
        SpeculativeMargin(5.0),
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 0, 0))),
        Transform::from_xyz(0.0, 1.0, 0.0),
        )
    );
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // and all entries provided by the crate:
}

fn player_movement(
    mut player_q: Query<&mut Transform, With<Player>>, // Player query (mutable)
    mut pan_orbit_q: Query<&mut PanOrbitState>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
     // Camera query (immutable, excluding Player)
) {
    let mut player_tfm = player_q.get_single_mut().expect("Error: Could not find a single player.");
    let mut pan_orbit_state = pan_orbit_q.get_single_mut().expect("Error: Could not find a single camera.");
    let mut camera_tfm  = camera_q.get_single_mut().expect("Error: Could not find a single camera.");
    
    // Get the rotation of the camera, and ignore pitch (up/down)
    let yaw = camera_tfm.rotation.to_euler(EulerRot::YXZ).0; // Extract yaw (rotation around Y-axis)

    // Compute the forward and right directions from the camera (ignoring vertical angle)
    let forward = Quat::from_rotation_y(yaw).mul_vec3(Vec3::NEG_Z).normalize();
    let right = Quat::from_rotation_y(yaw).mul_vec3(Vec3::X).normalize();

   
    // Initialize movement vector
    let mut movement = Vec3::ZERO;

    // Handle keyboard input for movement
    if keyboard_input.pressed(KeyCode::KeyW) {
        movement += forward;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement -= forward;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement -= right;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement += right;
    }

    // Normalize movement to prevent faster diagonal movement
    if movement != Vec3::ZERO {
        movement = movement.normalize();
    }

    // Apply movement with a speed factor
    let speed = 0.1;
    player_tfm.translation += movement * speed;

    // Update the camera's center to follow the player add right to off center player
    pan_orbit_state.center = player_tfm.translation+right*1.5; 
    camera_tfm.rotation = Quat::from_euler(EulerRot::YXZ, pan_orbit_state.yaw, pan_orbit_state.pitch, 0.0);
    camera_tfm.translation = pan_orbit_state.center + camera_tfm.back() * pan_orbit_state.radius;   
}












