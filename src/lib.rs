use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::camera::Projection,
};
use std::f32::consts::PI;

pub struct PanOrbitCameraPlugin;

impl Plugin for PanOrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(pan_orbit_camera);
    }
}

#[derive(Component)]
pub struct PanOrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

#[derive(Bundle)]
pub struct PanOrbitCameraBundle {
    #[bundle]
    camera_bundle: Camera3dBundle,
    controller: PanOrbitCamera,
}

impl PanOrbitCameraBundle {
    pub fn new(eye: Vec3, target: Vec3) -> Self {
        let radius = (eye - target).length();
        let transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);
        let camera_bundle = Camera3dBundle {
            transform,
            ..default()
        };

        let pan_orbit_camera = PanOrbitCamera {
            focus: target,
            radius,
            ..default()
        };

        Self {
            camera_bundle,
            controller: pan_orbit_camera,
        }
    }
}

fn pan_orbit_camera(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll: f32 = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }

    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            let up = transform.rotation * Vec3::Y;

            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;

        if rotation_move.length_squared() > 0.0 {
            any = true;

            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * PI * 2.0;

                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);

            transform.rotation = yaw * transform.rotation;
            transform.rotation = transform.rotation * pitch;
        } else if pan.length_squared() > 0.0 {
            any = true;

            let window = get_primary_window_size(&windows);

            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;

                let right = transform.rotation * Vec3::X * -pan.x;
                let up = transform.rotation * Vec3::Y * pan.y;
                let translation = (right + up) * pan_orbit.radius;

                pan_orbit.focus += translation;
            }
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            let rot_matrix = Mat3::from_quat(transform.rotation);

            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();

    Vec2::new(window.width() as f32, window.height() as f32)
}
