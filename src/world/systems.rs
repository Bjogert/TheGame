//! Systems for the world module.
use bevy::{
    ecs::message::MessageReader,
    input::{mouse::MouseMotion, ButtonInput},
    math::primitives::Plane3d,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

use crate::world::components::{FlyCamera, PrimarySun};

const GROUND_SCALE: f32 = 100.0;
const CAMERA_START_POS: Vec3 = Vec3::new(-12.0, 8.0, 16.0);

/// Spawns the initial scene: ground plane, light, and a fly camera.
pub fn spawn_world_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default()))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(90, 140, 90),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        })),
        Transform::from_scale(Vec3::splat(GROUND_SCALE)),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(16.0, 32.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
        PrimarySun,
    ));

    let mut camera_transform = Transform::from_translation(CAMERA_START_POS);
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);
    let (yaw, pitch) = yaw_pitch_from_transform(&camera_transform);

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        FlyCamera::new(yaw, pitch),
    ));
}

/// Toggles cursor grab when engaging the fly camera look mode.
pub fn update_cursor_grab(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if mouse_buttons.just_pressed(MouseButton::Right) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    } else if mouse_buttons.just_released(MouseButton::Right) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

/// Applies mouse look to the fly camera when the right mouse button is held.
pub fn fly_camera_mouse_look(
    mut motion_events: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    let mut cumulative_delta = Vec2::ZERO;
    for ev in motion_events.read() {
        cumulative_delta += ev.delta;
    }

    if !mouse_buttons.pressed(MouseButton::Right) {
        return;
    }

    if cumulative_delta == Vec2::ZERO {
        return;
    }

    if let Ok((mut fly_cam, mut transform)) = query.single_mut() {
        fly_cam.yaw -= cumulative_delta.x * fly_cam.look_sensitivity * time.delta_secs();
        fly_cam.pitch -= cumulative_delta.y * fly_cam.look_sensitivity * time.delta_secs();
        fly_cam.pitch = fly_cam.pitch.clamp(-1.54, 1.54);

        let rotation = Quat::from_axis_angle(Vec3::Y, fly_cam.yaw)
            * Quat::from_axis_angle(Vec3::X, fly_cam.pitch);
        transform.rotation = rotation.normalize();
    }
}

/// Moves the fly camera using WASD + Space/LShift.
pub fn fly_camera_translate(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&FlyCamera, &mut Transform)>,
) {
    if let Ok((fly_cam, mut transform)) = query.single_mut() {
        let mut direction = Vec3::ZERO;
        let forward = {
            let f = transform.forward().as_vec3();
            Vec3::new(f.x, 0.0, f.z).normalize_or_zero()
        };
        let right = {
            let r = transform.right().as_vec3();
            Vec3::new(r.x, 0.0, r.z).normalize_or_zero()
        };
        if keyboard.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction += -forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction += -right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += right;
        }
        if keyboard.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            direction += -Vec3::Y;
        }

        if direction.length_squared() > 0.0 {
            let modifier = if keyboard.pressed(KeyCode::ControlLeft) {
                2.5
            } else {
                1.0
            };
            transform.translation +=
                direction.normalize() * fly_cam.move_speed * modifier * time.delta_secs();
        }
    }
}

fn yaw_pitch_from_transform(transform: &Transform) -> (f32, f32) {
    let forward = -transform.forward().as_vec3();
    let yaw = forward.x.atan2(forward.z);
    let pitch = forward.y.asin();
    (yaw, pitch)
}
