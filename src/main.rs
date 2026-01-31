use bevy::prelude::*;
use bevy::window::WindowResolution;

mod components;
mod physics;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bounce Simulation".to_string(),
                resolution: WindowResolution::new(800, 800),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_camera, systems::setup_circle).chain())
        .add_systems(Update, (systems::move_circle, systems::check_collision))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
