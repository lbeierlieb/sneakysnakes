use bevy::{color::palettes::basic::*, prelude::*};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (512.0, 512.0).into(), // Fixed width and height
                title: "Fixed 2D Screen".to_string(),
                resizable: false,                   // Disable resizing
                ..default()
            }),
            ..default()
        }))
        .insert_state::<AppState>(AppState::MainMenu)
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        .add_systems(OnEnter(AppState::InGame), setup_in_game)
        .add_systems(OnExit(AppState::InGame), cleanup_in_game)
        .add_systems(Update, update_main_menu.run_if(in_state(AppState::MainMenu)))
        .add_systems(Update, game_logic.run_if(in_state(AppState::InGame)))
        .run();
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text2d::new("Press space to start"),
    ));
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, Or<(With<Camera>, With<Text2d>)>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn update_main_menu(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Space) {
        commands.set_state(AppState::InGame);
    }
}

fn setup_in_game (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(256.0, 256.0, 0.0)),
    ));
    commands.spawn((
        Player::new(Color::from(RED)),
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::from(YELLOW))),
        Transform::default().with_scale(Vec3::splat(5.)).with_translation(Vec3::new(10.,10.,0.)),
    ));
}

fn cleanup_in_game(mut commands: Commands, query: Query<Entity, Or<(With<Camera>, With<Mesh2d>)>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
struct Player {
    dir: Vec3,
    speed: f32,
    color: Color,
}

impl Player {
    fn new(color: Color) -> Self {
        Player {
            dir: Vec3::new(1., 0., 0.),
            speed: 100.0,
            color,
        }
    }
}

fn game_logic(mut query: Query<(&mut Transform, &mut Player)>, keyboard_input: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,) {
    for (mut transform, mut player) in &mut query {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            let rotation = Quat::from_rotation_z(std::f32::consts::PI / 30.0);
            player.dir = rotation.mul_vec3(player.dir);
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            let rotation = Quat::from_rotation_z(-std::f32::consts::PI / 30.0);
            player.dir = rotation.mul_vec3(player.dir);
        } else if keyboard_input.pressed(KeyCode::Escape) {
            commands.set_state(AppState::MainMenu);
        }
        transform.translation += player.dir * time.delta_secs() * player.speed;
        commands.spawn((
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(player.color))),
            Transform::default().with_scale(Vec3::splat(5.)).with_translation(Vec3::new(transform.translation.x, transform.translation.y, -1.0)),
    ));
    }
}
