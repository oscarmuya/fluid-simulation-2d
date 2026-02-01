use bevy::prelude::*;
use bevy::window::WindowResolution;

mod components;
mod physics;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fluid Simulation".to_string(),
                resolution: WindowResolution::new(800, 800),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_camera, systems::setup_particles).chain())
        .add_systems(
            Update,
            (
                systems::mouse_button_input,
                systems::move_particles,
                systems::particle_physics,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
