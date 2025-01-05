use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::{color::palettes::basic::*, prelude::*};
use rand::Rng;
use std::collections::HashSet;
use std::time::Duration;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    RoundActive,
    RoundOver,
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
        .add_systems(OnEnter(AppState::MainMenu), cleanup_in_game)
        .add_systems(
            OnEnter(AppState::MainMenu),
            setup_main_menu.after(cleanup_in_game),
        )
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        .add_systems(OnEnter(AppState::RoundActive), setup_in_game)
        .add_systems(OnEnter(AppState::RoundOver), setup_round_over)
        .add_systems(OnExit(AppState::RoundOver), cleanup_in_game)
        .add_systems(OnExit(AppState::RoundOver), cleanup_round_over)
        .add_systems(
            Update,
            update_main_menu.run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(
            Update,
            spawn_items.run_if(in_state(AppState::RoundActive)),
        )
        .add_systems(Update, game_logic.run_if(in_state(AppState::RoundActive)))
        .add_systems(
            Update,
            check_round_over
                .run_if(in_state(AppState::RoundActive))
                .after(game_logic),
        )
        .add_systems(
            Update,
            item_collection
                .run_if(in_state(AppState::RoundActive))
                .after(game_logic),
        )
        .add_systems(
            Update,
            update_round_over.run_if(in_state(AppState::RoundOver)),
        )
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
    mut query: Query<&mut Text2d>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.set_state(AppState::RoundActive);
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
    if let Ok(mut text) = query.get_single_mut() {
        text.0 = format!("Number of players: {}", settings.number_of_players);
    }
}

fn setup_round_over(mut commands: Commands, query: Query<&Player>) {
    let mut winner_name = None;
    for player in &query {
        if player.alive {
            winner_name = Some(player.name.clone());
        }
    }
    let text = match winner_name {
        Some(name) => format!("Player {} won!", name),
        None => "lol! Nobody won this round".to_string(),
    };
    commands.spawn((
        Text2d::new(text),
        Transform::from_translation(Vec3::new(1024., 1024., 2.)),
        TextFont {
            font_size: 130.0,
            ..default()
        },
    ));
}

fn update_round_over(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.set_state(AppState::RoundActive);
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.set_state(AppState::MainMenu);
    }
}

fn cleanup_round_over(mut commands: Commands, query: Query<Entity, With<Text2d>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
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
        Transform::from_translation(Vec3::new(1024.0, 1024.0, 0.0))
            .with_scale(Vec3::new(4., 4., 1.)),
    ));
    if settings.number_of_players >= 1 {
        spawn_player(
            "RED".to_string(),
            Color::from(RED),
            (KeyCode::ArrowLeft, KeyCode::ArrowRight),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }
    if settings.number_of_players >= 2 {
        spawn_player(
            "GREEN".to_string(),
            Color::from(GREEN),
            (KeyCode::KeyA, KeyCode::KeyD),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }
    if settings.number_of_players >= 3 {
        spawn_player(
            "BLUE".to_string(),
            Color::from(BLUE),
            (KeyCode::KeyV, KeyCode::KeyN),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    commands.insert_resource(ItemSpawnState::new());
}

fn spawn_player(
    name: String,
    color: Color,
    steer_keys: (KeyCode, KeyCode),
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let (position, direction) = random_position_and_direction();
    commands.spawn((
        Player::new(name, color, direction, steer_keys),
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::from(YELLOW))),
        Transform::default()
            .with_scale(Vec3::splat(20.))
            .with_translation(position),
    ));
}

fn random_position_and_direction() -> (Vec3, Vec3) {
    let mut rng = rand::thread_rng();

    let position = Vec3::new(
        rng.gen_range(100.0..1948.0),
        rng.gen_range(100.0..1948.0),
        0.,
    );

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
    name: String,
    dir: Vec3,
    speed: f32,
    color: Color,
    steer_keys: (KeyCode, KeyCode),
    alive: bool,
    gap_state: PlayerGapState,
}

impl Player {
    fn new(name: String, color: Color, dir: Vec3, steer_keys: (KeyCode, KeyCode)) -> Self {
        Player {
            name,
            dir,
            speed: 200.0,
            color,
            steer_keys,
            alive: true,
            gap_state: PlayerGapState::new(),
        }
    }
}

struct PlayerGapState {
    gapping: bool,
    timer: Timer,
}

impl PlayerGapState {
    fn new() -> Self {
        Self {
            gapping: false,
            timer: PlayerGapState::random_timer(),
        }
    }

    fn random_timer() -> Timer {
        let mut rng = rand::thread_rng();
        Timer::new(
            Duration::from_millis(rng.gen_range(1000..5000)),
            TimerMode::Once,
        )
    }

    fn gap_timer() -> Timer {
        Timer::new(Duration::from_millis(300), TimerMode::Once)
    }

    fn update(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if self.timer.finished() {
            if self.gapping {
                self.gapping = false;
                self.timer = PlayerGapState::random_timer();
            } else {
                self.gapping = true;
                self.timer = PlayerGapState::gap_timer();
            }
        }
    }
}

#[derive(Resource)]
struct TrailTexture {
    image_handle: Handle<Image>,
}

#[derive(Component)]
struct Item {

}

fn game_logic(
    mut query: Query<(&mut Transform, &mut Player)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    trail_texture: Res<TrailTexture>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.set_state(AppState::MainMenu);
    }

    for (mut transform, mut player) in &mut query {
        if !player.alive {
            continue;
        }

        let (left_key, right_key) = player.steer_keys;
        if keyboard_input.pressed(left_key) {
            let rotation = Quat::from_rotation_z(std::f32::consts::PI / 60.0);
            player.dir = rotation.mul_vec3(player.dir);
        }
        if keyboard_input.pressed(right_key) {
            let rotation = Quat::from_rotation_z(-std::f32::consts::PI / 60.0);
            player.dir = rotation.mul_vec3(player.dir);
        }

        let texture_handle = &trail_texture.image_handle;
        let texture = images.get_mut(texture_handle).unwrap();

        // Map the world position to texture space
        let size = texture.size().x as usize;

        let pos_before = transform.translation;
        let coords_before_update =
            get_all_coordinates_around(pos_before.x, pos_before.y, 10., size);

        transform.translation += player.dir * time.delta_secs() * player.speed;

        let pos_after = transform.translation;
        let coords_after_update = get_all_coordinates_around(pos_after.x, pos_after.y, 10., size);

        player.gap_state.update(time.delta());
        if !player.gap_state.gapping {
            let coords_to_draw = coords_before_update
                .difference(&coords_after_update)
                .collect::<HashSet<_>>();

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
        }

        for (x, y) in coords_after_update {
            let index = (y * size + x) * 4; // RGBA
            let alpha = texture.data[index + 3];
            if alpha != 0 {
                player.alive = false;
            }
        }

        if is_player_out_of_bounds(pos_after.x, pos_after.y, 10., size) {
            player.alive = false;
        }
    }
}

fn check_round_over(mut commands: Commands, query: Query<&Player>) {
    let mut players_alive = 0;
    for player in &query {
        if player.alive {
            players_alive += 1;
        }
    }
    if players_alive <= 1 {
        commands.set_state(AppState::RoundOver);
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

#[derive(Resource)]
struct ItemSpawnState {
    time_to_next_spawn: Timer,
}

impl ItemSpawnState {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let time_to_next_spawn = Timer::new(
            Duration::from_millis(rng.gen_range(1000..5000)),
            TimerMode::Once,
        );
        ItemSpawnState { time_to_next_spawn }
    }

    fn update(&mut self, delta: Duration) -> bool {
        self.time_to_next_spawn.tick(delta);
        if self.time_to_next_spawn.finished() {
            self.time_to_next_spawn = ItemSpawnState::random_timer();
            true
        } else {
            false
        }
    }

    fn random_timer() -> Timer {
        let mut rng = rand::thread_rng();
        Timer::new(
            Duration::from_millis(rng.gen_range(3000..7000)),
            TimerMode::Once,
        )
    }

    fn random_position() -> Vec3 {
        let mut rng = rand::thread_rng();

        Vec3::new(
            rng.gen_range(100.0..1948.0),
            rng.gen_range(100.0..1948.0),
            -3.,
        )
    }
}

fn spawn_items(mut commands: Commands, mut spawn_state: ResMut<ItemSpawnState>, time: Res<Time>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    if spawn_state.update(time.delta()) {
        commands.spawn((
            Item {},
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(GREEN))),
            Transform::default()
                .with_scale(Vec3::splat(150.))
                .with_translation(ItemSpawnState::random_position()),
        ));
    }
}

fn item_collection(mut commands: Commands, mut player_query: Query<(&mut Player, &Transform)>, item_query: Query<(Entity, &Item, &Transform)>) {
    for (mut player, player_transform) in &mut player_query {
        let player_translation = player_transform.translation;
        let player_xy = Vec2::new(player_translation.x, player_translation.y);

        for (entity, item, item_transform) in &item_query {
            let item_translation = item_transform.translation;
            let item_xy = Vec2::new(item_translation.x, item_translation.y);

            if player_xy.distance(item_xy) <= 85. {
                commands.entity(entity).despawn();
            }
        }
    }
}
