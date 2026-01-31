use crate::{
    components::{Force, Particle, ParticleMaterial, Velocity},
    physics::{poly6_kernel, spiky_kernel_gradient},
};
use bevy::prelude::*;

const GRAVITY: f32 = 50.0;
const COLLISION_DAMPING: f32 = 0.9;

const SUBSTEPS: u32 = 10;

const RADIUS: f32 = 5.0;
const VELOCITY: f32 = 0.0;
const MASS: f32 = 100.0;

const GRID_SIZE: f32 = 30.0;
const SPACING: f32 = 0.0;

const SMOOTHING_RADIUS_H: f32 = 100.0;
const PRESSURE_MULTIPLIER: f32 = 50.0;
const REST_DENSITY: f32 = 0.6;

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

pub fn check_particle_collision_1(
    mut query: Query<
        (
            &mut Transform,
            &mut Particle,
            &mut Velocity,
            &mut ParticleMaterial,
        ),
        With<Particle>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut list: Vec<_> = query.iter_mut().collect();

    for i in 0..list.len() {
        let (left, right) = list.split_at_mut(i + 1);
        let (t_i, p_i, v_i, m_i) = &mut left[i];

        let mut density_i = 0.0;

        for (t_j, p_j, v_j, _) in right {
            let dx = t_j.translation.x - t_i.translation.x;
            let dy = t_j.translation.y - t_i.translation.y;

            let m_2 = dx * dx + dy * dy;
            let radii = p_i.radius + p_j.radius;
            let distance = (dx * dx + dy * dy).sqrt();

            println!("{}", distance);

            // calxulate density and pressure of p_i
            let smoothing_kernel = poly6_kernel(distance, SMOOTHING_RADIUS_H);
            density_i += p_j.mass * smoothing_kernel;

            if radii * radii >= m_2 {
                // collision normal
                let nx = dx / distance;
                let ny = dy / distance;

                // relative velocity
                let dvx = v_i.x - v_j.x;
                let dvy = v_i.y - v_j.y;

                // relative v in collision normal direction
                let dvn = dvx * nx + dvy * ny;

                if dvn > 0.0 {
                    v_i.x -= dvn * nx;
                    v_i.y -= dvn * ny;

                    v_j.x += dvn * nx;
                    v_j.y += dvn * ny;
                }
            }
        }

        let pressure_i = PRESSURE_MULTIPLIER * (density_i - REST_DENSITY);
        p_i.density = density_i;
        p_i.pressure = pressure_i;

        if density_i > 0.0 {
            println!("{}", density_i);
        }

        if let Some(mat) = materials.get_mut(&m_i.0) {
            if density_i > 0.6 {
                mat.color = Color::srgb(1.0, 0.0, 0.0);
            } else if density_i > 0.4 {
                mat.color = Color::srgb(1.0, 0.5, 0.0);
            } else {
                mat.color = Color::srgb(0.0, 0.45, 0.7);
            }
        }
    }
}

pub fn check_particle_collision(
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
) {
    let mut list: Vec<_> = query.iter_mut().collect();

    // First pass: density
    for i in 0..list.len() {
        let pos_i = list[i].0.translation;
        let mut density_i = 0.0;
        for j in 0..list.len() {
            if i == j {
                continue;
            }
            let dx = list[j].0.translation.x - pos_i.x;
            let dy = list[j].0.translation.y - pos_i.y;
            let distance = (dx * dx + dy * dy).sqrt();
            density_i += list[j].1.mass * poly6_kernel(distance, SMOOTHING_RADIUS_H);
        }
        list[i].1.density = density_i;
        list[i].1.pressure = (PRESSURE_MULTIPLIER * (density_i - REST_DENSITY)).max(0.0);

        if let Some(mat) = materials.get_mut(&list[i].3.0) {
            if density_i > 0.75 {
                mat.color = Color::srgb(1.0, 0.0, 0.0);
            } else if density_i > 0.55 {
                mat.color = Color::srgb(1.0, 0.5, 0.0);
            } else {
                mat.color = Color::srgb(0.0, 0.45, 0.7);
            }
        }
    }

    // pressure force
    for i in 0..list.len() {
        let mut force_x = 0.0;
        let mut force_y = 0.0;

        let pos_i = list[i].0.translation;

        for j in 0..list.len() {
            if i == j {
                continue;
            }
            let dx = list[j].0.translation.x - pos_i.x;
            let dy = list[j].0.translation.y - pos_i.y;
            let distance = (dx * dx + dy * dy).sqrt();

            let (gx, gy) = spiky_kernel_gradient(dx, dy, distance, SMOOTHING_RADIUS_H);

            let pressure_avg = (list[i].1.pressure + list[j].1.pressure) / 2.0;
            let mass_ratio = list[j].1.mass / list[j].1.density.max(0.0001);

            force_x += mass_ratio * pressure_avg * gx;
            force_y += mass_ratio * pressure_avg * gy;
        }

        // Apply force
        let fx = -list[i].1.mass * force_x;
        let fy = -list[i].1.mass * force_y;

        if fx.abs() > 0.5 && fy.abs() < 0.01 {
            // This should be an interior particle with outward force
            println!(
                "Interior particle {}: pos ({}, {}), fx: {}, vel: {}",
                i,
                list[i].0.translation.x,
                list[i].0.translation.y,
                fx,
                list[i].4.x // velocity after force applied
            );
        }

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

    let dt = time.delta_secs() / SUBSTEPS as f32;

    for _ in 0..SUBSTEPS {
        for (mut transform, mut velocity, particle, force) in &mut query {
            let r = particle.radius;
            let half_w = window_width / 2.0;
            let half_h = window_height / 2.0;

            let acceleration = (force.x / particle.mass, force.y / particle.mass);
            velocity.x += acceleration.0 * dt;
            velocity.y += acceleration.1 * dt;

            // velocity.y -= GRAVITY * dt;

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
}
