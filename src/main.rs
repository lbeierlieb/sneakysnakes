use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::{color::palettes::basic::*, prelude::*};
use rand::Rng;
use std::collections::HashSet;

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
    let size = 2048;
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
            translation: Vec3::new(1024.0, 1024.0, -2.0), // Position in the middle of the camera's view
            scale: Vec3::new(1., -1., 1.),
            ..Default::default()
        },
    ));
    commands.insert_resource(TrailTexture {
        image_handle: texture_handle,
    });

    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(1024.0, 1024.0, 0.0)).with_scale(Vec3::new(4., 4., 1.)),
    ));
    if settings.number_of_players >= 1 {
        spawn_player(
            Color::from(RED),
            (KeyCode::ArrowLeft, KeyCode::ArrowRight),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }
    if settings.number_of_players >= 2 {
        spawn_player(
            Color::from(GREEN),
            (KeyCode::KeyA, KeyCode::KeyD),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }
    if settings.number_of_players >= 3 {
        spawn_player(
            Color::from(BLUE),
            (KeyCode::KeyV, KeyCode::KeyN),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }
}

fn spawn_player(
    color: Color,
    steer_keys: (KeyCode, KeyCode),
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let (position, direction) = random_position_and_direction();
    commands.spawn((
        Player::new(color, direction, steer_keys),
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::from(YELLOW))),
        Transform::default()
            .with_scale(Vec3::splat(20.))
            .with_translation(position),
    ));
}

fn random_position_and_direction() -> (Vec3, Vec3) {
    let mut rng = rand::thread_rng();

    let position = Vec3::new(rng.gen_range(0.0..512.0), rng.gen_range(0.0..512.0), 0.);

    let direction = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.).normalize();

    (position, direction)
}

fn cleanup_in_game(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Camera>, With<Mesh2d>, With<Sprite>)>>,
    mut images: ResMut<Assets<Image>>,
    trail_texture: Res<TrailTexture>,
) {
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
    steer_keys: (KeyCode, KeyCode),
    alive: bool,
}

impl Player {
    fn new(color: Color, dir: Vec3, steer_keys: (KeyCode, KeyCode)) -> Self {
        Player {
            dir,
            speed: 200.0,
            color,
            steer_keys,
            alive: true,
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
        if !player.alive {
            continue;
        }

        let (left_key, right_key) = player.steer_keys;
        if keyboard_input.pressed(left_key) {
            let rotation = Quat::from_rotation_z(std::f32::consts::PI / 60.0);
            player.dir = rotation.mul_vec3(player.dir);
        } else if keyboard_input.pressed(right_key) {
            let rotation = Quat::from_rotation_z(-std::f32::consts::PI / 60.0);
            player.dir = rotation.mul_vec3(player.dir);
        } else if keyboard_input.just_pressed(KeyCode::Escape) {
            commands.set_state(AppState::MainMenu);
        }

        let texture_handle = &trail_texture.image_handle;
        let texture = images.get_mut(texture_handle).unwrap();

        // Map the world position to texture space
        let size = texture.size().x as usize;

        let pos_before = transform.translation;
        let coords_before_update = get_all_coordinates_around(pos_before.x, pos_before.y, 10., size);

        transform.translation += player.dir * time.delta_secs() * player.speed;

        let pos_after = transform.translation;
        let coords_after_update = get_all_coordinates_around(pos_after.x, pos_after.y, 10., size);

        let coords_to_draw = coords_before_update.difference(&coords_after_update).collect::<HashSet<_>>();

        for (x, y) in coords_to_draw {
            let index = (y * size + x) * 4; // RGBA
            let color = player.color.to_srgba();
            texture.data[index..index + 4].copy_from_slice(&[
                (color.red * 255.) as u8,
                (color.green * 255.) as u8,
                (color.blue * 255.) as u8,
                (color.alpha * 255.) as u8,
            ]);
        }

        if is_player_out_of_bounds(pos_after.x, pos_after.y, 10., size) {
            player.alive = false;
        }
    }
}

fn get_all_coordinates_around(x: f32, y: f32, r: f32, size: usize) -> HashSet<(usize, usize)> {
    let ux = x as usize;
    let uy = y as usize;
    let ur = r as usize;

    let start_x = ux.saturating_sub(ur + 1).clamp(0, size - 1);
    let end_x = (ux + ur + 1).clamp(0, size - 1);

    let start_y = uy.saturating_sub(ur + 1).clamp(0, size - 1);
    let end_y = (uy + ur + 1).clamp(0, size - 1);

    (start_x..end_x)
        .flat_map(|x| (start_y..end_y).map(move |y| (x, y)))
        .filter(|&(px, py)| Vec2::new(px as f32, py as f32).distance(Vec2::new(x, y)) <= r)
        .collect()
}

fn is_player_out_of_bounds(x: f32, y: f32, r: f32, size: usize) -> bool {
    let size = size as f32;
    x < r || x > size - r || y < r || y > size - r
}
