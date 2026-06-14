use bevy::asset::RenderAssetUsages;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::CompositeAlphaMode;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const ITERATIONS_PER_FRAME: usize = 200000;

#[derive(Clone, Copy)]
struct AffineTransform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
    color: [f32; 3],
}

#[derive(Resource)]
struct FractalConfig {
    transforms: [AffineTransform; 4],
    zoom: f32,
}

#[derive(Resource)]
struct FractalState {
    x: f32,
    y: f32,
    c_r: f32,
    c_g: f32,
    c_b: f32,
    seed: u64,
    histogram: Vec<(u32, f32, f32, f32)>,
    max_density: u32,
    image_handle: Handle<Image>,
}

// Marker component for our FPS Text
#[derive(Component)]
struct FpsText;

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
        // 1. Add the diagnostics plugin to track frame rate
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (render_fractal, update_fps_text)) // 2. Register the update system
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

    // 3. Spawn the UI Text element to display the FPS
    commands.spawn((
        Text::new("FPS: "),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        FpsText,
    ));

    // Insert the dynamic configuration for the shapes and colors
    commands.insert_resource(FractalConfig {
        zoom: 1.2,
        transforms: [
            // Transform 0: Vibrant Red
            AffineTransform { a: 0.5, b: 0.0, c: 0.0, d: 0.0, e: 0.5, f: 0.5, color: [1.0, 0.1, 0.2] },
            // Transform 1: Cyan
            AffineTransform { a: 0.5, b: 0.0, c: -0.5, d: 0.0, e: 0.5, f: -0.5, color: [0.1, 0.8, 1.0] },
            // Transform 2: Purple
            AffineTransform { a: 0.5, b: 0.0, c: 0.5, d: 0.0, e: 0.5, f: -0.5, color: [0.6, 0.1, 1.0] },
            // Transform 3: Gold (The Rotator)
            AffineTransform { a: 0.4, b: -0.4, c: 0.0, d: 0.4, e: 0.4, f: 0.0, color: [1.0, 0.8, 0.1] },
        ],
    });

    commands.insert_resource(FractalState {
        x: 0.0,
        y: 0.0,
        c_r: 1.0,
        c_g: 1.0,
        c_b: 1.0,
        seed: 123456789,
        histogram: vec![(0, 0.0, 0.0, 0.0); (WIDTH * HEIGHT) as usize],
        max_density: 1,
        image_handle,
    });
}

// 4. System to update the text with the latest FPS value
fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the text string with the smoothed FPS value
                text.0 = format!("FPS: {:.1}", value);
            }
        }
    }
}

fn render_fractal(
    mut state: ResMut<FractalState>,
    mut config: ResMut<FractalConfig>,
    mut images: ResMut<Assets<Image>>,
) {
    // Animate 'a' across all transforms
    for transform in &mut config.transforms {
        transform.a += 0.001; 
    }

    // Fade the canvas instead of clearing it (creates motion trails)
    let mut local_max_density = 1; 
    
    for entry in state.histogram.iter_mut() {
        let fade_factor = 0.90; 

        // Decay the density and the color sums
        entry.0 = (entry.0 as f32 * fade_factor) as u32;
        entry.1 *= fade_factor;
        entry.2 *= fade_factor;
        entry.3 *= fade_factor;

        // Clean up floating point dust and calculate the max density locally
        if entry.0 == 0 {
            entry.1 = 0.0;
            entry.2 = 0.0;
            entry.3 = 0.0;
        } else if entry.0 > local_max_density {
            local_max_density = entry.0;
        }
    }

    // Assign the new max density back to the state now that the loop is over
    state.max_density = local_max_density;

    let mut x = state.x;
    let mut y = state.y;
    let mut c_r = state.c_r;
    let mut c_g = state.c_g;
    let mut c_b = state.c_b;
    let mut seed = state.seed;

    for _ in 0..ITERATIONS_PER_FRAME {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r = ((seed >> 32) % 4) as usize; 

        // Fetch the chosen transform
        let transform = &config.transforms[r];

        // Smoothly pull the point's color towards the transform's base color
        c_r = (c_r + transform.color[0]) * 0.5;
        c_g = (c_g + transform.color[1]) * 0.5;
        c_b = (c_b + transform.color[2]) * 0.5;

        // Affine Step (Linear structure)
        let nx = transform.a * x + transform.b * y + transform.c;
        let ny = transform.d * x + transform.e * y + transform.f;

        // Non-Linear Step (The Chaos - Swirl Variation)
        let r2 = nx * nx + ny * ny;
        let sin_r = r2.sin();
        let cos_r = r2.cos();

        x = nx * sin_r - ny * cos_r;
        y = nx * cos_r + ny * sin_r;

        // Map back to screen space using the configurable zoom
        let px = ((x / config.zoom + 1.0) * 0.5 * WIDTH as f32) as i32;
        let py = ((y / config.zoom + 1.0) * 0.5 * HEIGHT as f32) as i32;

        if px >= 0 && px < WIDTH as i32 && py >= 0 && py < HEIGHT as i32 {
            let index = (py as u32 * WIDTH + px as u32) as usize;
            
            // Scope the mutable borrow of the histogram
            let new_density = {
                let entry = &mut state.histogram[index];
                entry.0 += 1;
                entry.1 += c_r;
                entry.2 += c_g;
                entry.3 += c_b;
                entry.0
            };

            if new_density > state.max_density {
                state.max_density = new_density;
            }
        }
    }

    state.x = x;
    state.y = y;
    state.c_r = c_r;
    state.c_g = c_g;
    state.c_b = c_b;
    state.seed = seed;

    // Render to Texture
    if let Some(image) = images.get_mut(&state.image_handle) {
        let max_d = state.max_density as f32;

        if let Some(data) = &mut image.data {
            for (i, &(density, r_sum, g_sum, b_sum)) in state.histogram.iter().enumerate() {
                let pixel_idx = i * 4;

                // If density is 0, we MUST clear the pixel so old frames don't stick around
                if density == 0 {
                    data[pixel_idx] = 0;
                    data[pixel_idx + 1] = 0;
                    data[pixel_idx + 2] = 0;
                    data[pixel_idx + 3] = 0; 
                    continue;
                }

                // Standard logarithmic tone mapping for brightness
                let brightness = (density as f32).ln() / max_d.ln();
                let intensity = brightness.powf(0.8);

                // Average out the accumulated colors
                let r_avg = r_sum / density as f32;
                let g_avg = g_sum / density as f32;
                let b_avg = b_sum / density as f32;

                // Multiply average color by the calculated intensity
                data[pixel_idx] = (r_avg * intensity * 255.0).clamp(0.0, 255.0) as u8;
                data[pixel_idx + 1] = (g_avg * intensity * 255.0).clamp(0.0, 255.0) as u8;
                data[pixel_idx + 2] = (b_avg * intensity * 255.0).clamp(0.0, 255.0) as u8;
                data[pixel_idx + 3] = 255;
            }
        }
    }
}
