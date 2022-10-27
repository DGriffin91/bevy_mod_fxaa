//! Shows how to render simple primitive shapes with a single color.

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use bevy_mod_fxaa::{FXAAPlugin, FXAA};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 1 })
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
                hdr: false, // Should work with and without hdr
                ..default()
            },
            ..default()
        })
        .insert(FXAA { enabled: true });

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
}

fn toggle_fxaa(keys: Res<Input<KeyCode>>, mut query: Query<&mut FXAA>, mut msaa: ResMut<Msaa>) {
    if keys.just_pressed(KeyCode::Key1) {
        for mut fxaa in &mut query {
            fxaa.enabled = false;
        }
        msaa.samples = 1;
    } else if keys.just_pressed(KeyCode::Key2) {
        for mut fxaa in &mut query {
            fxaa.enabled = false;
        }
        msaa.samples = 4;
    } else if keys.just_pressed(KeyCode::Key3) {
        for mut fxaa in &mut query {
            fxaa.enabled = true;
        }
        msaa.samples = 1;
    }
}
