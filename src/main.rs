use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::WindowResized;
use bevy::{color::palettes::basic::*, prelude::*};
use rand::Rng;
use std::collections::HashSet;
use std::time::Duration;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    RoundStart,
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

#[derive(Resource)]
struct WindowSize {
    width: f32,
    height: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize {
            width: 512.,
            height: 512.,
        }
    }
}

impl WindowSize {
    fn get_smallest_dimension(&self) -> f32 {
        if self.width < self.height {
            self.width
        } else {
            self.height
        }
    }
}

fn main() {
    let window_size = WindowSize::default();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (window_size.width, window_size.height).into(),
                title: "Fixed 2D Screen".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .insert_state::<AppState>(AppState::MainMenu)
        .insert_resource(GameSettings::default())
        .insert_resource(window_size)
        .add_systems(Update, on_resize_system)
        .add_systems(OnEnter(AppState::MainMenu), cleanup_in_game)
        .add_systems(
            OnEnter(AppState::MainMenu),
            setup_main_menu.after(cleanup_in_game),
        )
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        .add_systems(OnEnter(AppState::RoundStart), setup_in_game)
        .add_systems(
            OnEnter(AppState::RoundStart),
            move_players_a_bit.after(setup_in_game),
        )
        .add_systems(OnEnter(AppState::RoundOver), setup_round_over)
        .add_systems(OnExit(AppState::RoundOver), cleanup_in_game)
        .add_systems(OnExit(AppState::RoundOver), cleanup_round_over)
        .add_systems(
            Update,
            update_round_start.run_if(in_state(AppState::RoundStart)),
        )
        .add_systems(
            Update,
            update_main_menu.run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(Update, spawn_items.run_if(in_state(AppState::RoundActive)))
        .add_systems(
            Update,
            update_player_item_effects.run_if(in_state(AppState::RoundActive)),
        )
        .add_systems(
            Update,
            game_logic
                .run_if(in_state(AppState::RoundActive))
                .after(update_player_item_effects),
        )
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
    commands.spawn((
        Text2d::new("hello"),
        Transform::from_translation(Vec3::new(0., 0., 2.)).with_scale(Vec3::new(
            1. / 512.,
            1. / 512.,
            1.,
        )),
        TextFont {
            font_size: 40.0,
            ..default()
        },
    ));
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<Text2d>>) {
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
        commands.set_state(AppState::RoundStart);
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
        Transform::from_translation(Vec3::new(0., 0., 2.)).with_scale(Vec3::new(
            1. / 512.,
            1. / 512.,
            1.,
        )),
        TextFont {
            font_size: 40.0,
            ..default()
        },
    ));
}

fn on_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut window_size: ResMut<WindowSize>,
    mut commands: Commands,
    query: Option<Single<Entity, With<Camera>>>,
) {
    if let Some(entity) = query.map(|single| single.into_inner()) {
        commands.entity(entity).despawn();
    }

    for e in resize_reader.read() {
        window_size.width = e.width;
        window_size.height = e.height;
    }

    let smallest_dim = window_size.get_smallest_dimension();
    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::new(
            2. / smallest_dim,
            2. / smallest_dim,
            1.,
        )),
    ));
}

fn update_round_start(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.set_state(AppState::RoundActive);
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.set_state(AppState::MainMenu);
    }
}

fn update_round_over(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.set_state(AppState::RoundStart);
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
    window_size: Res<WindowSize>,
) {
    let smallest_dim = window_size.get_smallest_dimension();
    let texture_size = if smallest_dim > 512. {
        smallest_dim as u32
    } else {
        512
    };
    let texture = Image::new_fill(
        Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &(0..(texture_size * texture_size * 4))
            .map(|i| if i % 4 == 3 { 0xff } else { 0 })
            .collect::<Vec<_>>(),
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
            translation: Vec3::new(0.0, 0.0, -2.0), // Position in the middle of the camera's view
            scale: Vec3::new(2. / texture_size as f32, 2. / texture_size as f32, 1.),
            ..Default::default()
        },
    ));
    commands.insert_resource(TrailTexture {
        image_handle: texture_handle,
    });

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

fn move_players_a_bit(
    query: Query<(&Transform, &Player)>,
    mut images: ResMut<Assets<Image>>,
    trail_texture: Res<TrailTexture>,
) {
    let texture_handle = &trail_texture.image_handle;
    let texture = images.get_mut(texture_handle).unwrap();

    for (transform, player) in &query {
        let pos = transform.translation;
        let pos_before = pos - player.dir * 10. / 256.;

        draw_trail(
            pos_before,
            player.dir,
            pos,
            player.dir,
            texture,
            player.color,
        );
    }
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
            .with_scale(Vec3::splat(5. / 256.))
            .with_translation(position),
    ));
}

fn random_position_and_direction() -> (Vec3, Vec3) {
    let mut rng = rand::thread_rng();

    let position = Vec3::new(rng.gen_range(-0.8..0.8), rng.gen_range(-0.8..0.8), 0.);

    let direction = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.).normalize();

    (position, direction)
}

fn cleanup_in_game(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Mesh2d>, With<Sprite>)>>,
    mut images: ResMut<Assets<Image>>,
    trail_texture: Option<Res<TrailTexture>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    if let Some(trail_texture) = trail_texture {
        images.remove(&trail_texture.image_handle);
    }
}

#[derive(Component)]
struct Player {
    name: String,
    dir: Vec3,
    color: Color,
    steer_keys: (KeyCode, KeyCode),
    alive: bool,
    gap_state: PlayerGapState,
    item_effects: Vec<(ItemEffectIndividual, Timer)>,
}

impl Player {
    fn new(name: String, color: Color, dir: Vec3, steer_keys: (KeyCode, KeyCode)) -> Self {
        Player {
            name,
            dir,
            color,
            steer_keys,
            alive: true,
            gap_state: PlayerGapState::new(),
            item_effects: Vec::new(),
        }
    }

    fn speed_mod(&self) -> i64 {
        let count_speed = self
            .item_effects
            .iter()
            .filter(|(effect, _)| *effect == ItemEffectIndividual::Speed)
            .count();
        let count_slow = self
            .item_effects
            .iter()
            .filter(|(effect, _)| *effect == ItemEffectIndividual::Slowness)
            .count();

        count_speed as i64 - count_slow as i64
    }

    fn update_item_effects(&mut self, delta: Duration) {
        let mut indices_to_remove = vec![];
        for (index, tuple) in self.item_effects.iter_mut().enumerate() {
            tuple.1.tick(delta);
            if tuple.1.finished() {
                indices_to_remove.push(index);
            }
        }
        for index in indices_to_remove.into_iter().rev() {
            self.item_effects.remove(index);
        }
    }

    fn add_effect(&mut self, effect: ItemEffectIndividual) {
        self.item_effects
            .push((effect, Timer::new(Duration::from_secs(5), TimerMode::Once)));
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
    effect: ItemEffectIndividual,
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

        let dir_before = player.dir;
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

        // Map the worjd position to texture space
        let size = texture.size().x as usize;

        let pos_before = transform.translation;

        let player_base_speed = 50. / 256.;
        let modifier = player.speed_mod() as f32 * 0.5 + 1.;
        let player_speed = player_base_speed * modifier;
        transform.translation += player.dir * time.delta_secs() * player_speed;

        for vec in get_collision_points(transform.translation, player.dir) {
            if let Some((x, y)) = game_to_texture_coord(vec, size) {
                let index = (y * size + x) * 4; // RGBA
                if texture.data[index] != 0
                    || texture.data[index + 1] != 0
                    || texture.data[index + 2] != 0
                {
                    // something was hit
                    player.alive = false;
                }
            } else {
                // player is out of bounds
                player.alive = false;
            }
        }

        player.gap_state.update(time.delta());
        if !player.gap_state.gapping {
            draw_trail(
                pos_before,
                dir_before,
                transform.translation,
                player.dir,
                texture,
                player.color,
            );
        }
    }
}

fn game_to_texture_vec(game_coord: Vec3, texture_size: usize) -> Vec3 {
    let x = (game_coord.x + 1.0) * texture_size as f32 / 2.0;
    let y = (game_coord.y - 1.0) * texture_size as f32 / -2.0;

    Vec3::new(x, y, game_coord.z)
}

fn game_to_texture_coord(game_coord: Vec3, texture_size: usize) -> Option<(usize, usize)> {
    let mapped_vec = game_to_texture_vec(game_coord, texture_size);
    let ix = mapped_vec.x as isize;
    let iy = mapped_vec.y as isize;

    if ix < 0 || ix >= texture_size as isize || iy < 0 || iy >= texture_size as isize {
        return None;
    }

    Some((ix as usize, iy as usize))
}

fn get_collision_points(translation: Vec3, dir: Vec3) -> Vec<Vec3> {
    let rotation_left = Quat::from_rotation_z(std::f32::consts::PI / 3.);
    let rotation_right = Quat::from_rotation_z(-std::f32::consts::PI / 3.);
    let front = translation + 2.5 / 256.0 * dir;
    let left = translation + 2.5 / 256.0 * rotation_left.mul_vec3(dir);
    let right = translation + 2.5 / 256.0 * rotation_right.mul_vec3(dir);
    vec![front, left, right]
}

fn draw_trail(
    translation_before: Vec3,
    dir_before: Vec3,
    translation_now: Vec3,
    dir_now: Vec3,
    texture: &mut Image,
    color: Color,
) {
    let size = texture.size().x as usize;
    let rotation_90deg = Quat::from_rotation_z(std::f32::consts::PI / 2.);

    let dir_rot_before = rotation_90deg.mul_vec3(dir_before);
    let left_before = translation_before + 2.5 / 256. * dir_rot_before;
    let right_before = translation_before - 2.5 / 256. * dir_rot_before;

    let dir_rot_now = rotation_90deg.mul_vec3(dir_now);
    let left_now = translation_now + 2.5 / 256. * dir_rot_now;
    let right_now = translation_now - 2.5 / 256. * dir_rot_now;

    let quad = [
        game_to_texture_vec(left_now, size),
        game_to_texture_vec(right_now, size),
        game_to_texture_vec(left_before, size),
        game_to_texture_vec(right_before, size),
    ];

    let coords_to_draw = get_all_coordinates_in_quad(quad);
    for (x, y) in coords_to_draw {
        let index = (y * size + x) * 4; // RGBA
        let color = color.to_srgba();
        texture.data[index..index + 4].copy_from_slice(&[
            (color.red * 255.) as u8,
            (color.green * 255.) as u8,
            (color.blue * 255.) as u8,
            (color.alpha * 255.) as u8,
        ]);
    }
}

fn check_round_over(mut commands: Commands, query: Query<&Player>) {
    let mut players_alive = 0;
    if query.iter().count() == 1 {
        return;
    }
    for player in &query {
        if player.alive {
            players_alive += 1;
        }
    }
    if players_alive <= 1 {
        commands.set_state(AppState::RoundOver);
    }
}

fn get_all_coordinates_in_quad(quad: [Vec3; 4]) -> HashSet<(usize, usize)> {
    let x_min = quad.iter().map(|v| v.x as usize).min().unwrap();
    let y_min = quad.iter().map(|v| v.y as usize).min().unwrap();
    let x_max = quad.iter().map(|v| v.x as usize).max().unwrap();
    let y_max = quad.iter().map(|v| v.y as usize).max().unwrap();

    let mut points = HashSet::new();

    for x in x_min..=x_max {
        for y in y_min..=y_max {
            let vec = Vec3::new(x as f32, y as f32, 0.);
            if is_point_inside_of_quad(vec, quad) {
                points.insert((x, y));
            }
        }
    }

    points
}

fn is_point_inside_of_quad(p: Vec3, quad: [Vec3; 4]) -> bool {
    let tria0 = [quad[0], quad[1], quad[2]];
    let tria1 = [quad[1], quad[2], quad[3]];

    is_point_inside_of_triangle(p, tria0) || is_point_inside_of_triangle(p, tria1)
}

fn is_point_inside_of_triangle(p: Vec3, mut triangle: [Vec3; 3]) -> bool {
    let area = triangle_area(triangle);
    let mut parts_area = 0.0;
    for i in 0..triangle.len() {
        let replaced = triangle[i];
        triangle[i] = p;
        parts_area += triangle_area(triangle);
        triangle[i] = replaced;
    }

    parts_area / area < 1.05
}

fn triangle_area(triangle: [Vec3; 3]) -> f32 {
    0.5 * (triangle[0].x * triangle[1].y
        + triangle[1].x * triangle[2].y
        + triangle[2].x * triangle[0].y
        - triangle[0].y * triangle[1].x
        - triangle[1].y * triangle[2].x
        - triangle[2].y * triangle[0].x)
        .abs()
}

#[derive(Resource)]
struct ItemSpawnState {
    time_to_next_spawn: Timer,
}

impl ItemSpawnState {
    fn new() -> Self {
        ItemSpawnState {
            time_to_next_spawn: ItemSpawnState::random_timer(),
        }
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
            Duration::from_millis(rng.gen_range(5000..15000)),
            TimerMode::Once,
        )
    }

    fn random_position() -> Vec3 {
        let mut rng = rand::thread_rng();

        Vec3::new(rng.gen_range(-0.8..0.8), rng.gen_range(-0.8..0.8), -3.)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ItemEffectIndividual {
    Speed,
    Slowness,
}

impl ItemEffectIndividual {
    fn get_random() -> Self {
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..2) {
            0 => Self::Speed,
            1 => Self::Slowness,
            _ => panic!("item randomizer is broken"),
        }
    }

    fn get_text(&self) -> String {
        match self {
            Self::Speed => "fast",
            Self::Slowness => "slow",
        }
        .to_string()
    }
}

fn spawn_items(
    mut commands: Commands,
    mut spawn_state: ResMut<ItemSpawnState>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if spawn_state.update(time.delta()) {
        let effect = ItemEffectIndividual::get_random();
        let entity = commands
            .spawn((
                Item { effect },
                Mesh2d(meshes.add(Circle::default())),
                MeshMaterial2d(materials.add(Color::srgba(0., 0., 0., 0.))),
                Transform::default()
                    .with_translation(ItemSpawnState::random_position())
                    .with_scale(Vec3::splat(1.)),
            ))
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(effect.get_text()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                Transform::from_translation(Vec3::new(0., 0., 0.2))
                    .with_scale(Vec3::splat(1. / 256.)),
            ));
            parent.spawn((
                Mesh2d(meshes.add(Circle::default())),
                MeshMaterial2d(materials.add(Color::from(GREEN))),
                Transform::from_translation(Vec3::new(0., 0., 0.1))
                    .with_scale(Vec3::splat(40. / 256.)),
            ));
        });
    }
}

fn update_player_item_effects(mut query: Query<&mut Player>, time: Res<Time>) {
    for mut player in &mut query {
        player.update_item_effects(time.delta());
    }
}

fn item_collection(
    mut commands: Commands,
    mut player_query: Query<(&mut Player, &Transform)>,
    item_query: Query<(Entity, &Item, &Transform)>,
) {
    for (mut player, player_transform) in &mut player_query {
        let player_translation = player_transform.translation;
        let player_xy = Vec2::new(player_translation.x, player_translation.y);

        for (entity, item, item_transform) in &item_query {
            let item_translation = item_transform.translation;
            let item_xy = Vec2::new(item_translation.x, item_translation.y);

            if player_xy.distance(item_xy) <= 22.5 / 256. {
                player.add_effect(item.effect);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
