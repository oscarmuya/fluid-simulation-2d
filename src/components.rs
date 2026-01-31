use bevy::prelude::*;

#[derive(Component)]
pub struct Particle {
    pub radius: f32,
    pub pressure: f32,
    pub density: f32,
    pub mass: f32,
}

#[derive(Component)]
pub struct ParticleMaterial(pub Handle<ColorMaterial>);

#[derive(Component, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug)]
pub struct Force {
    pub x: f32,
    pub y: f32,
}
