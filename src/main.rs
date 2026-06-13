use bevy::asset::RenderAssetUsages; 
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::CompositeAlphaMode;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const ITERATIONS_PER_FRAME: usize = 100;

#[derive(Resource)]
struct FractalState {
    x: f32,
    y: f32,
    seed: u64,
    histogram: Vec<u32>,
    max_density: u32,
    image_handle: Handle<Image>,
}

fn main() {
    App::new()
        // Make the camera clear with a transparent background
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WIDTH, HEIGHT).into(),
                title: "Fractal Flame Software Renderer".into(),
                // Enable window transparency at the OS level
                transparent: true,
                decorations: false,
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, render_fractal)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);

    let image = Image::new_fill(
        Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        // Fill initially with completely transparent black pixels
        &[0, 0, 0, 0],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let image_handle = images.add(image);

    commands.spawn(Sprite {
        image: image_handle.clone(),
        custom_size: Some(Vec2::new(WIDTH as f32, HEIGHT as f32)),
        ..default()
    });

    commands.insert_resource(FractalState {
        x: 0.0,
        y: 0.0,
        seed: 123456789,
        histogram: vec![0; (WIDTH * HEIGHT) as usize],
        max_density: 1,
        image_handle,
    });
}

fn render_fractal(mut state: ResMut<FractalState>, mut images: ResMut<Assets<Image>>) {
    let mut x = state.x;
    let mut y = state.y;
    let mut seed = state.seed;

    for _ in 0..ITERATIONS_PER_FRAME {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r = (seed >> 32) % 3;

        match r {
            0 => {
                x = x * 0.5;
                y = y * 0.5 + 0.5;
            }
            1 => {
                x = x * 0.5 - 0.5;
                y = y * 0.5 - 0.5;
            }
            _ => {
                x = x * 0.5 + 0.5;
                y = y * 0.5 - 0.5;
            }
        }

        let px = ((x + 1.0) * 0.5 * WIDTH as f32) as i32;
        let py = ((y + 1.0) * 0.5 * HEIGHT as f32) as i32;

        if px >= 0 && px < WIDTH as i32 && py >= 0 && py < HEIGHT as i32 {
            let index = (py as u32 * WIDTH + px as u32) as usize;
            state.histogram[index] += 1;

            if state.histogram[index] > state.max_density {
                state.max_density = state.histogram[index];
            }
        }
    }

    state.x = x;
    state.y = y;
    state.seed = seed;

    if let Some(image) = images.get_mut(&state.image_handle) {
        let max_d = state.max_density as f32;

        if let Some(data) = &mut image.data {
            for (i, &density) in state.histogram.iter().enumerate() {
                if density == 0 {
                    continue;
                }

                let brightness = (density as f32).ln() / max_d.ln();
                let color_val = (brightness * 255.0).min(255.0) as u8;

                let pixel_idx = i * 4;

                data[pixel_idx] = color_val; // Red
                data[pixel_idx + 1] = color_val / 2; // Green
                data[pixel_idx + 2] = color_val / 4; // Blue
                data[pixel_idx + 3] = 255; // Alpha
            }
        }
    }
}
