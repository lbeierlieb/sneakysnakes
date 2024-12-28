use bevy::{color::palettes::basic::{PURPLE,BLUE}, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_all_transforms)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform::default().with_scale(Vec3::splat(128.)).with_translation(Vec3::new(50.,200.,1.)),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::from(BLUE))),
        Transform::default().with_scale(Vec3::splat(256.)),
    ));
}

fn rotate_all_transforms(mut query: Query<&mut Transform, With<Mesh2d>>, time: Res<Time>) {
    for mut transform in &mut query {
        let elapsed_time = time.elapsed_secs();
        transform.translation.y = 100.0 * f32::sin(elapsed_time);
    }
}
