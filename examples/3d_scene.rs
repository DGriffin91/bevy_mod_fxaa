use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{
        render_resource::{Extent3d, SamplerDescriptor, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
};

use bevy_mod_fxaa::{FXAAPlugin, Quality, FXAA};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(FXAAPlugin)
        .add_startup_system(setup)
        .add_system(toggle_fxaa);

    app.run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    for i in 0..5 {
        // cube
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.25 })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(images.add(uv_debug_texture())),
                ..default()
            }),
            transform: Transform::from_xyz(i as f32 * 0.25 - 1.0, 0.125, -i as f32 * 0.5),
            ..default()
        });
    }

    // FlightHelmet
    commands.spawn(SceneBundle {
        scene: asset_server.load("FlightHelmet/FlightHelmet.gltf#Scene0"),
        ..default()
    });

    // light
    const HALF_SIZE: f32 = 2.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            PI * -0.15,
            PI * -0.15,
        )),
        ..default()
    });

    // camera
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                hdr: false, // Works with and without hdr
                ..default()
            },
            transform: Transform::from_xyz(0.7, 0.7, 1.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        })
        .insert(FXAA::default());

    println!("Toggle with:\n1 - NO AA\n2 - MSAA 4\n3 - FXAA (default)");
    println!("Threshold:\n7 - LOW\n8 - MEDIUM\n9 - HIGH (default)\n0 - ULTRA");
}

fn toggle_fxaa(keys: Res<Input<KeyCode>>, mut query: Query<&mut FXAA>, mut msaa: ResMut<Msaa>) {
    for mut fxaa in &mut query {
        if keys.just_pressed(KeyCode::Key1) {
            fxaa.enabled = false;
            msaa.samples = 1;
        } else if keys.just_pressed(KeyCode::Key2) {
            fxaa.enabled = false;
            msaa.samples = 4;
        } else if keys.just_pressed(KeyCode::Key3) {
            fxaa.enabled = true;
            msaa.samples = 1;
        } else if keys.just_pressed(KeyCode::Key7) {
            fxaa.edge_threshold = Quality::Low;
            fxaa.edge_threshold_min = Quality::Low;
        } else if keys.just_pressed(KeyCode::Key8) {
            fxaa.edge_threshold = Quality::Medium;
            fxaa.edge_threshold_min = Quality::Medium;
        } else if keys.just_pressed(KeyCode::Key9) {
            fxaa.edge_threshold = Quality::High;
            fxaa.edge_threshold_min = Quality::High;
        } else if keys.just_pressed(KeyCode::Key0) {
            fxaa.edge_threshold = Quality::Ultra;
            fxaa.edge_threshold_min = Quality::Ultra;
        }
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    let mut img = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    );
    img.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor::default());
    img
}
