use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::CompositeAlphaMode;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
// CRITICAL: Bumped from 10 to 50,000. Chaos needs volume to render quickly.
const ITERATIONS_PER_FRAME: usize = 10000;

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
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WIDTH, HEIGHT).into(),
                title: "Fractal Flame Software Renderer".into(),
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

        // Increased to 4 to accommodate the new transformation
        let r = (seed >> 32) % 4;

        // 1. Affine Step (Linear structure)
        let nx;
        let ny;

        match r {
            0 => {
                nx = x * 0.5;
                ny = y * 0.5 + 0.5;
            }
            1 => {
                nx = x * 0.5 - 0.5;
                ny = y * 0.5 - 0.5;
            }
            2 => {
                nx = x * 0.5 + 0.5;
                ny = y * 0.5 - 0.5;
            }
            // New Transform: Rotate and scale to break the triangle
            _ => {
                nx = x * 0.4 - y * 0.4;
                ny = x * 0.4 + y * 0.4;
            }
        }

        // 2. Non-Linear Step (The Chaos)
        // Applying a "Swirl" variation
        let r2 = nx * nx + ny * ny;
        let sin_r = r2.sin();
        let cos_r = r2.cos();

        x = nx * sin_r - ny * cos_r;
        y = nx * cos_r + ny * sin_r;

        // Map back to screen space. We zoom out by dividing by 1.2
        // because the swirl can push coordinates further outwards.
        let zoom_factor = 1.2;
        let px = ((x / zoom_factor + 1.0) * 0.5 * WIDTH as f32) as i32;
        let py = ((y / zoom_factor + 1.0) * 0.5 * HEIGHT as f32) as i32;

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

                // powf(0.8) sharpens the falloff, creating brighter hot spots
                let color_val = (brightness.powf(0.8) * 255.0).min(255.0) as u8;

                let pixel_idx = i * 4;

                data[pixel_idx] = color_val; // Red
                data[pixel_idx + 1] = (color_val as f32 * 0.6) as u8; // Green
                data[pixel_idx + 2] = (color_val as f32 * 0.2) as u8; // Blue
                data[pixel_idx + 3] = 255; // Alpha
            }
        }
    }
}
