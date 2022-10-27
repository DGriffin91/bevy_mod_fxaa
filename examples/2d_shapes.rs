use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use bevy_mod_fxaa::{FXAAPlugin, Quality, FXAA};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FXAAPlugin)
        .add_startup_system(setup)
        .add_system(toggle_fxaa)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                hdr: false, // Works with and without hdr
                ..default()
            },
            ..default()
        })
        .insert(FXAA::default());

    // Rectangle
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(50.0, 100.0)),
            ..default()
        },
        ..default()
    });

    // Circle
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::from_translation(Vec3::new(-100., 0., 0.)),
        ..default()
    });

    // Hexagon
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
        material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
        transform: Transform::from_translation(Vec3::new(100., 0., 0.)),
        ..default()
    });

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
