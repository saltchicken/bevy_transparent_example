use bevy::prelude::*;
use bevy::window::CompositeAlphaMode;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                transparent: true,
                decorations: false,
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn(Sprite {
        color: Color::srgb(1.0, 0.0, 0.0),
        custom_size: Some(Vec2::new(100.0, 100.0)),
        ..default()
    });
}
