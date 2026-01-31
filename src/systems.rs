use crate::components::{Particle, Velocity};
use bevy::prelude::*;

const GRAVITY: f32 = 100.0;
const COLLISION_DAMPING: f32 = 0.9;

const RADIUS: f32 = 5.0;
const VELOCITY: f32 = 100.0;

// grid
const GRID_SIZE: f32 = 50.0;
const SPACING: f32 = 5.0;

pub fn setup_circle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let point = (GRID_SIZE / 2.0 * (RADIUS * 2.0 + SPACING * 2.0)) as i32;

    let mesh_handle = meshes.add(Circle::new(RADIUS));

    for x in (-point..=point).step_by((RADIUS * 2.0 + SPACING * 2.0) as usize) {
        for y in (-point..=point).step_by((RADIUS * 2.0 + SPACING * 2.0) as usize) {
            commands.spawn((
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
                Transform::from_xyz(x as f32, y as f32, 0.0),
                Particle { radius: RADIUS },
                Velocity {
                    x: VELOCITY,
                    y: VELOCITY,
                },
            ));
        }
    }
}

pub fn check_collision(
    mut query: Query<(&mut Transform, &Particle, &mut Velocity), With<Particle>>,
) {
    let mut list: Vec<_> = query.iter_mut().collect();

    for i in 0..list.len() {
        let (left, right) = list.split_at_mut(i + 1);
        let (t_i, p_i, v_i) = &mut left[i];

        for (t_j, p_j, v_j) in right {
            let dx = t_j.translation.x - t_i.translation.x;
            let dy = t_j.translation.y - t_i.translation.y;

            let m_2 = dx * dx + dy * dy;

            let radii = p_i.radius + p_j.radius;

            if radii * radii >= m_2 {
                let distance = (dx * dx + dy * dy).sqrt();

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
    }
}

pub fn move_circle(
    mut query: Query<(&mut Transform, &mut Velocity, &Particle), With<Particle>>,
    time: Res<Time>,
    window: Single<&Window>,
) {
    let window_width = window.width();
    let window_height = window.height();

    for (mut transform, mut velocity, particle) in &mut query {
        let r = particle.radius;
        let half_w = window_width / 2.0;
        let half_h = window_height / 2.0;

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

        velocity.y -= GRAVITY * time.delta_secs();

        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}
