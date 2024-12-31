use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::{color::palettes::basic::*, prelude::*};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    InGame,
}

#[derive(Resource)]
struct GameSettings {
    number_of_players: u8,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            number_of_players: 2,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (512.0, 512.0).into(), // Fixed width and height
                title: "Fixed 2D Screen".to_string(),
                resizable: false, // Disable resizing
                ..default()
            }),
            ..default()
        }))
        .insert_state::<AppState>(AppState::MainMenu)
        .insert_resource(GameSettings::default())
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        .add_systems(OnEnter(AppState::InGame), setup_in_game)
        .add_systems(OnExit(AppState::InGame), cleanup_in_game)
        .add_systems(
            Update,
            update_main_menu.run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(Update, game_logic.run_if(in_state(AppState::InGame)))
        .run();
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((Text2d::new("Press space to start"),));
}

fn cleanup_main_menu(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Camera>, With<Text2d>)>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn update_main_menu(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<GameSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.set_state(AppState::InGame);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        if settings.number_of_players > 1 {
            settings.number_of_players -= 1;
        }
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        if settings.number_of_players < 6 {
            settings.number_of_players += 1;
        }
    }
    println!("{}", settings.number_of_players);
}

fn setup_in_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    settings: Res<GameSettings>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = 512;
    let texture = Image::new_fill(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &vec![0; (size * size * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    let texture_handle = images.add(texture);
    commands.spawn((
        Sprite {
            image: texture_handle.clone(),
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(256.0, 256.0, -2.0), // Position in the middle of the camera's view
            scale: Vec3::new(1., -1., 1.),
            ..Default::default()
        },
    ));
    commands.insert_resource(TrailTexture {
        image_handle: texture_handle,
    });

    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(256.0, 256.0, 0.0)),
    ));
    for i in 0..settings.number_of_players {
        commands.spawn((
            Player::new(Color::from(RED)),
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(YELLOW))),
            Transform::default()
                .with_scale(Vec3::splat(5.))
                .with_translation(Vec3::new(10., i as f32 * 10., 0.)),
        ));
    }
}

fn cleanup_in_game(mut commands: Commands, query: Query<Entity, Or<(With<Camera>, With<Mesh2d>, With<Sprite>)>>, mut images: ResMut<Assets<Image>>, trail_texture: Res<TrailTexture>,) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    images.remove(&trail_texture.image_handle);
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
#[derive(Resource)]
struct TrailTexture {
    image_handle: Handle<Image>,
}

fn game_logic(
    mut query: Query<(&mut Transform, &mut Player)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    trail_texture: Res<TrailTexture>,
) {
    for (mut transform, mut player) in &mut query {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            let rotation = Quat::from_rotation_z(std::f32::consts::PI / 30.0);
            player.dir = rotation.mul_vec3(player.dir);
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            let rotation = Quat::from_rotation_z(-std::f32::consts::PI / 30.0);
            player.dir = rotation.mul_vec3(player.dir);
        } else if keyboard_input.just_pressed(KeyCode::Escape) {
            commands.set_state(AppState::MainMenu);
        }
        transform.translation += player.dir * time.delta_secs() * player.speed;

        let texture_handle = &trail_texture.image_handle;
        let texture = images.get_mut(texture_handle).unwrap();

        let dot_position = transform.translation;

        // Map the world position to texture space
        let size = texture.size().x as usize;
        let x = (dot_position.x as usize).clamp(0, size - 1);
        let y = (dot_position.y as usize).clamp(0, size - 1);

        let index = (y * size + x) * 4; // RGBA
        let color = player.color.to_srgba();
        texture.data[index..index + 4].copy_from_slice(&[
            (color.red * 255.) as u8,
            (color.green * 255.) as u8,
            (color.blue * 255.) as u8,
            (color.alpha * 255.) as u8,
        ]);
    }
}
