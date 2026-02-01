use crate::{
    components::{Force, Particle, ParticleMaterial, Velocity},
    physics::{poly6_kernel, spiky_kernel_gradient},
};
use bevy::{platform::collections::HashMap, prelude::*, window::PrimaryWindow};

const GRAVITY: f32 = 60.0;
const COLLISION_DAMPING: f32 = 0.7;

const RADIUS: f32 = 4.0;
const VELOCITY: f32 = 0.0;
const MASS: f32 = 100.0;

const GRID_SIZE: f32 = 30.0;
const SPACING: f32 = 1.0;

const SMOOTHING_RADIUS_H: f32 = 20.0;
const PRESSURE_MULTIPLIER: f32 = 1000.0;
const REST_DENSITY: f32 = 0.8;

const MOUSE_RADIUS: f32 = 100.0;
const MOUSE_STRENGTH: f32 = 2000.0;

pub fn setup_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let point = (GRID_SIZE / 2.0 * (RADIUS * 2.0 + SPACING * 2.0)) as i32;

    let mesh_handle = meshes.add(Circle::new(RADIUS));

    for x in (-point..=point).step_by((RADIUS * 2.0 + SPACING * 2.0) as usize) {
        for y in (-point..=point).step_by((RADIUS * 2.0 + SPACING * 2.0) as usize) {
            let material = materials.add(Color::srgb(0.0, 0.45, 0.7));
            commands.spawn((
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(x as f32, y as f32, 0.0),
                ParticleMaterial(material),
                Particle {
                    radius: RADIUS,
                    pressure: 0.0,
                    density: 0.0,
                    mass: MASS,
                },
                Velocity {
                    x: VELOCITY,
                    y: VELOCITY,
                },
                Force { x: 0.0, y: 0.0 },
            ));
        }
    }
}

pub fn particle_physics(
    mut query: Query<
        (
            &mut Transform,
            &mut Particle,
            &mut Velocity,
            &mut ParticleMaterial,
            &mut Force,
        ),
        With<Particle>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,

    time: Res<Time>,
) {
    let mut list: Vec<_> = query.iter_mut().collect();
    let dt = time.delta_secs();

    // predict positions
    // This is very crucial because it makes the particles not go to a position where
    // the density would be higher.
    let mut predicted_positions = Vec::with_capacity(list.len());

    for item in list.iter() {
        let current_pos = item.0.translation;
        let velocity = &item.2;

        // Prediction: pos + vel * dt
        let predicted = Vec3 {
            x: current_pos.x + velocity.x * dt,
            y: current_pos.y + velocity.y * dt,
            z: 0.0,
        };
        predicted_positions.push(predicted);
    }

    // spatial hashing
    let mut spatial_hash: HashMap<(i32, i32), Vec<usize>> = HashMap::default();
    let cell_size = SMOOTHING_RADIUS_H;

    for (i, pos) in predicted_positions.iter().enumerate() {
        let key_x = (pos.x / cell_size).floor() as i32;
        let key_y = (pos.y / cell_size).floor() as i32;
        spatial_hash.entry((key_x, key_y)).or_default().push(i);
    }

    // First pass: calculate density and pressure
    for i in 0..list.len() {
        let pos_i = predicted_positions[i];
        let mut density_i = 0.0;

        let grid_x = (pos_i.x / cell_size).floor() as i32;
        let grid_y = (pos_i.y / cell_size).floor() as i32;

        // find the neighbors
        for x_off in -1..=1 {
            for y_off in -1..=1 {
                let key = (grid_x + x_off, grid_y + y_off);
                if let Some(neighbors) = spatial_hash.get(&key) {
                    for &j in neighbors {
                        let pos_j = predicted_positions[j];

                        let dx = pos_j.x - pos_i.x;
                        let dy = pos_j.y - pos_i.y;
                        let distance_sq = dx * dx + dy * dy;

                        if distance_sq >= cell_size * cell_size {
                            continue;
                        }

                        let distance = distance_sq.sqrt();
                        let p6k = poly6_kernel(distance, SMOOTHING_RADIUS_H);

                        // We access the mass from the original list
                        density_i += list[j].1.mass * p6k;
                    }
                }
            }
        }

        list[i].1.density = density_i;
        list[i].1.pressure = PRESSURE_MULTIPLIER * (density_i - REST_DENSITY);

        if let Some(mat) = materials.get_mut(&list[i].3.0) {
            if density_i > 0.7 {
                mat.color = Color::srgb(1.0, 0.0, 0.0);
            } else if density_i > 0.45 {
                mat.color = Color::srgb(1.0, 0.5, 0.0);
            } else {
                mat.color = Color::srgb(0.0, 0.45, 0.7);
            }
        }
    }

    // Second pass: calculate pressure force
    for i in 0..list.len() {
        let mut force_x = 0.0;
        let mut force_y = 0.0;

        let pos_i = predicted_positions[i];

        // find the neighbors
        let grid_x = (pos_i.x / cell_size).floor() as i32;
        let grid_y = (pos_i.y / cell_size).floor() as i32;

        for x_off in -1..=1 {
            for y_off in -1..=1 {
                let key = (grid_x + x_off, grid_y + y_off);

                if let Some(neighbors) = spatial_hash.get(&key) {
                    for &j in neighbors {
                        if i == j {
                            continue;
                        }

                        let pos_j = predicted_positions[j];
                        let dx = pos_j.x - pos_i.x;
                        let dy = pos_j.y - pos_i.y;
                        let distance_sq = dx * dx + dy * dy;

                        if distance_sq >= cell_size * cell_size {
                            continue;
                        }

                        let distance = distance_sq.sqrt();

                        let (gx, gy) = spiky_kernel_gradient(dx, dy, distance, SMOOTHING_RADIUS_H);

                        let pressure_avg = (list[i].1.pressure + list[j].1.pressure) / 2.0;
                        let mass_ratio = list[j].1.mass / list[j].1.density;

                        force_x += mass_ratio * pressure_avg * gx;
                        force_y += mass_ratio * pressure_avg * gy;
                    }
                }
            }
        }

        // Apply force
        let fx = -list[i].1.mass * force_x;
        let fy = -list[i].1.mass * force_y;

        list[i].4.x = fx;
        list[i].4.y = fy;
    }
}

pub fn move_particles(
    mut query: Query<(&mut Transform, &mut Velocity, &Particle, &Force), With<Particle>>,
    time: Res<Time>,
    window: Single<&Window>,
) {
    let window_width = window.width();
    let window_height = window.height();

    let dt = time.delta_secs();

    for (mut transform, mut velocity, particle, force) in &mut query {
        let r = particle.radius;
        let half_w = window_width / 2.0;
        let half_h = window_height / 2.0;

        let acceleration = (force.x / particle.mass, force.y / particle.mass);
        velocity.x += acceleration.0 * dt;
        velocity.y += acceleration.1 * dt;

        velocity.y -= GRAVITY * dt;

        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;

        // check horizontally
        if transform.translation.x >= half_w - r {
            transform.translation.x = half_w - r;
            velocity.x *= -COLLISION_DAMPING;
        } else if transform.translation.x <= -half_w + r {
            transform.translation.x = -half_w + r;
            velocity.x *= -COLLISION_DAMPING;
        }

        // check vertically
        if transform.translation.y >= half_h - r {
            transform.translation.y = half_h - r;
            velocity.y *= -COLLISION_DAMPING;
        } else if transform.translation.y <= -half_h + r {
            transform.translation.y = -half_h + r;
            velocity.y *= -COLLISION_DAMPING;
        }
    }
}

pub fn mouse_button_input(
    mut query: Query<(&Transform, &mut Force, &Velocity), With<Particle>>,
    window: Single<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let (camera, camera_transform) = q_camera.single().unwrap();

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        let is_pushing = buttons.pressed(MouseButton::Left);
        let is_pulling = buttons.pressed(MouseButton::Right);

        if !is_pushing && !is_pulling {
            return;
        }

        let direction_multiplier = if is_pushing { 1.0 } else { -1.0 };

        for (transform, mut force, velocity) in query.iter_mut() {
            let pos = transform.translation.truncate(); // Vec3 -> Vec2
            let mouse_pos = world_position;

            let diff = pos - mouse_pos;
            let distance_sq = diff.length_squared();

            if distance_sq < MOUSE_RADIUS * MOUSE_RADIUS {
                let distance = distance_sq.sqrt();

                let dir = if distance > 0.001 {
                    diff / distance
                } else {
                    Vec2::ZERO
                };

                // "Linear Falloff": Force is strong at center, 0 at edge
                let percent = 1.0 - (distance / MOUSE_RADIUS);

                // Calculate vector force
                let strength = percent * direction_multiplier;

                force.x += dir.x * (strength - velocity.x) * MOUSE_STRENGTH;
                force.y += dir.y * (strength - velocity.y) * MOUSE_STRENGTH;
            }
        }
    }
}
