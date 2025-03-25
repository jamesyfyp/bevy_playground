use avian3d::{math::*, prelude::*};
use bevy::prelude::*;

//largely https://github.com/Jondolf/avian/blob/main/crates/avian3d/examples/kinematic_character_3d/plugin.rs

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
  fn build(&self, app: &mut App) {
    app.add_event::<MovementAction>().add_systems(
      Update,
      (
        keyboard_input,
        update_grounded,
        movement,
        apply_gravity,
        apply_movement_damping,
      )
        .in_set(PhysicsSet::Prepare),
    );
  }
}

// Movement action event
#[derive(Event, Debug)]
pub enum MovementAction {
  Move(Vector2),
  Jump,
}

/// The gravitational acceleration used for a character controller.
#[derive(Component, Reflect)]
pub struct ControllerGravity(Vector);

/// A marker component indicating that an entity is using a character controller.
#[derive(Component, Reflect)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Grounded;

// Component for movement properties
#[derive(Component, Reflect)]
pub struct MovementAcceleration(pub f32);

#[derive(Component, Reflect)]
pub struct MovementDampingFactor(pub f32);

#[derive(Component, Reflect)]
pub struct JumpImpulse(pub f32);
/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component, Reflect)]
pub struct MaxSlopeAngle(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
  character_controller: CharacterController,
  rigid_body: RigidBody,
  collider: Collider,
  ground_caster: ShapeCaster,
  locked_axes: LockedAxes,
  gravity: ControllerGravity,
  movement: MovementBundle,
}

impl MovementBundle {
  pub const fn new(
    acceleration: Scalar,
    damping: Scalar,
    jump_impulse: Scalar,
    max_slope_angle: Scalar,
  ) -> Self {
    Self {
      acceleration: MovementAcceleration(acceleration),
      damping: MovementDampingFactor(damping),
      jump_impulse: JumpImpulse(jump_impulse),
      max_slope_angle: MaxSlopeAngle(max_slope_angle),
    }
  }
}

impl Default for MovementBundle {
  fn default() -> Self {
    Self::new(30.0, 0.9, 7.0, PI * 0.45)
  }
}

impl CharacterControllerBundle {
  pub fn new(collider: Collider, gravity: Vector) -> Self {
    // Create shape caster as a slightly smaller version of collider
    let mut caster_shape = collider.clone();
    caster_shape.set_scale(Vector::ONE * 0.99, 10);

    Self {
      character_controller: CharacterController,
      rigid_body: RigidBody::Dynamic,
      collider,
      ground_caster: ShapeCaster::new(
        caster_shape,
        Vector::ZERO,
        Quaternion::default(),
        Dir3::NEG_Y,
      )
      .with_max_distance(0.2),
      gravity: ControllerGravity(gravity),
      locked_axes: LockedAxes::ROTATION_LOCKED,
      movement: MovementBundle::default(),
    }
  }

  pub fn with_movement(
    mut self,
    acceleration: Scalar,
    damping: Scalar,
    jump_impulse: Scalar,
    max_slope_angle: Scalar,
  ) -> Self {
    self.movement = MovementBundle::new(acceleration, damping, jump_impulse, max_slope_angle);
    self
  }
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
  acceleration: MovementAcceleration,
  damping: MovementDampingFactor,
  jump_impulse: JumpImpulse,
  max_slope_angle: MaxSlopeAngle,
}

// Keyboard input system
fn keyboard_input(
  mut movement_event_writer: EventWriter<MovementAction>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  camera_q: Query<&Transform, With<Camera>>,
) {
  let camera_tfm = camera_q.single();
  let yaw = camera_tfm.rotation.to_euler(EulerRot::YXZ).0;

  let forward = Quat::from_rotation_y(yaw).mul_vec3(Vec3::NEG_Z).normalize();
  let right = Quat::from_rotation_y(yaw).mul_vec3(Vec3::X).normalize();

  let mut movement = Vec3::ZERO;

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

  if movement != Vec3::ZERO {
    movement = movement.normalize();
  }

  movement_event_writer.send(MovementAction::Move(Vector2::new(movement.x, movement.z)));

  if keyboard_input.just_pressed(KeyCode::Space) {
    movement_event_writer.send(MovementAction::Jump);
  }
}

fn update_grounded(
  mut commands: Commands,
  mut query: Query<
    (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
    With<CharacterController>,
  >,
) {
  for (entity, hits, rotation, max_slope_angle) in &mut query {
    // The character is grounded if the shape caster has a hit with a normal
    // that isn't too steep.
    let is_grounded = hits.iter().any(|hit| {
      if let Some(angle) = max_slope_angle {
        (rotation * -hit.normal2).angle_between(Vector::Y).abs() <= angle.0
      } else {
        true
      }
    });

    if is_grounded {
      commands.entity(entity).insert(Grounded);
    } else {
      commands.entity(entity).remove::<Grounded>();
    }
  }
}

// Movement system
fn movement(
  time: Res<Time>,
  mut movement_event_reader: EventReader<MovementAction>,
  mut controllers: Query<(
    &MovementAcceleration,
    &JumpImpulse,
    &mut LinearVelocity,
    Has<Grounded>,
  )>,
) {
  let delta_time = time.delta().as_secs_f32();

  for event in movement_event_reader.read() {
    for (movement_acceleration, jump_impulse, mut linear_velocity, is_grounded) in &mut controllers
    {
      match event {
        MovementAction::Move(direction) => {
          let movement_force = Vec3::new(direction.x, 0.0, direction.y) * movement_acceleration.0;
          linear_velocity.x += movement_force.x * delta_time;
          linear_velocity.z += movement_force.z * delta_time;
        }
        MovementAction::Jump => {
          if is_grounded {
            linear_velocity.y = jump_impulse.0;
          }
        }
      }
    }
  }
}

// Apply damping to prevent infinite sliding
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
  for (damping_factor, mut linear_velocity) in &mut query {
    linear_velocity.x *= damping_factor.0;
    linear_velocity.z *= damping_factor.0;
  }
}

//apply gravity
fn apply_gravity(
  time: Res<Time>,
  mut controllers: Query<(&ControllerGravity, &mut LinearVelocity)>,
) {
  // Precision is adjusted so that the example works with
  // both the `f32` and `f64` features. Otherwise you don't need this.
  let delta_time = time.delta_secs_f64().adjust_precision();

  for (gravity, mut linear_velocity) in &mut controllers {
    linear_velocity.0 += gravity.0 * delta_time;
  }
}
