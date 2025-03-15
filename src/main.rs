use std::f32;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use getrandom::getrandom;

#[derive(Resource)]
struct ScreenDimensions {
    width: f32,
    height: f32,
}

impl Default for ScreenDimensions {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Resource)]
struct GameArea {
    width: f32,
    height: f32,
    margin: f32,
}

impl Default for GameArea {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            margin: 0.0,
        }
    }
}

#[derive(Resource)]
struct SpawnMode {
    mouse_control: bool,
    object_count: u32,
}

impl Default for SpawnMode {
    fn default() -> Self {
        Self {
            mouse_control: false,
            object_count: 0,
        }
    }
}

#[derive(Component)]
struct FallingCube;

#[derive(Component)]
struct ClearButton;

#[derive(Component)]
struct ToggleButton;

#[derive(Component)]
struct DelayedDeletion {
    timer: Timer,
}

#[derive(Component)]
struct StationaryTimer {
    timer: Timer,
    is_moving: bool,
}

#[derive(Component)]
struct MovingGround {
    speed: f32,
    direction: f32,
    bounds: f32,
}

#[derive(Component)]
struct RespawnCube;

fn main() {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            resolution: (800., 600.).into(),
            ..default()
        }),
        ..default()
    };

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(window_plugin));
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());

    #[cfg(debug_assertions)]
    app.add_plugins(RapierDebugRenderPlugin::default());

    app.init_resource::<ScreenDimensions>()
        .init_resource::<GameArea>()
        .init_resource::<SpawnMode>()
        .add_systems(Startup, (
            setup_camera,
            spawn_ground,
            spawn_walls,  // Add this
            spawn_box_collider,
            spawn_clear_button,
            spawn_toggle_button
        ))
        .add_systems(Update, (
            update_game_area,
            update_screen_dimensions,
            throw_cubes,
            check_bottom_despawn,
            handle_clear_button,
            handle_toggle_button,
            move_ground,
            check_respawn_cube,
        ))
        .add_systems(Update, check_delayed_deletion)
        .add_systems(Update, check_stationary_objects)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_ground(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.7, 0.2),
            custom_size: Some(Vec2::new(600.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -300.0, 0.0),
        Visibility::default(),
        RigidBody::Fixed,
        Collider::cuboid(300.0, 25.0),
        MovingGround {
            speed: 100.0,
            direction: 1.0,
            bounds: 300.0,
        },
    ));
}

fn spawn_walls(mut commands: Commands) {
    // Left wall
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.7),
            custom_size: Some(Vec2::new(50.0, 800.0)),
            ..default()
        },
        Transform::from_xyz(-600.0, 0.0, 0.0),
        Visibility::default(),
        RigidBody::Fixed,
        Collider::cuboid(25.0, 400.0),
    ));

    // Right wall
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.7),
            custom_size: Some(Vec2::new(50.0, 800.0)),
            ..default()
        },
        Transform::from_xyz(600.0, 0.0, 0.0),
        Visibility::default(),
        RigidBody::Fixed,
        Collider::cuboid(25.0, 400.0),
    ));
}

fn spawn_box_collider(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.5, 1.0),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
        RigidBody::Dynamic,
        Collider::cuboid(25.0, 25.0),
        RespawnCube,
    ));
}

const NORMAL_BUTTON: Color = Color::srgb(0.2, 0.2, 0.2);

fn spawn_clear_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Button,
        ClearButton,
        Interaction::default(),
        bevy_ui::Node {
            width: Val::Px(150.0),
            height: Val::Px(65.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::BLACK),
        BorderRadius::MAX,
        BackgroundColor(NORMAL_BUTTON),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Clear"),
            TextFont {
                font: asset_server.load("fonts/JetBrainsMono-Regular.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
    });
}

fn spawn_toggle_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Button,
        ToggleButton,
        Interaction::default(),
        bevy_ui::Node {
            width: Val::Px(150.0),
            height: Val::Px(65.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        },
        BorderColor(Color::BLACK),
        BorderRadius::MAX,
        BackgroundColor(NORMAL_BUTTON),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Toggle Mode"),
            TextFont {
                font: asset_server.load("fonts/JetBrainsMono-Regular.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
    });
}

fn random_f32() -> f32 {
    let mut buf = [0u8; 4];
    getrandom(&mut buf).unwrap();
    let random_bits = u32::from_le_bytes(buf);
    (random_bits as f32) / (u32::MAX as f32)
}

fn random_range(min: f32, max: f32) -> f32 {
    min + random_f32() * (max - min)
}

fn clamp_bounds(x: f32, y: f32, width: f32, height: f32) -> (f32, f32) {
    let clamped_x = x.clamp(-width / 2.0, width / 2.0);
    let clamped_y = y.clamp(-height / 2.0, height / 2.0);
    (clamped_x, clamped_y)
}

fn throw_cubes(
    mut commands: Commands,
    time: Res<Time>,
    mut last_spawn: Local<f32>,
    game_area: Res<GameArea>,
    windows: Query<&Window>,
    mut spawn_mode: ResMut<SpawnMode>,
) {
    let now = time.elapsed_secs();
    if now - *last_spawn > 0.5 {
        *last_spawn = now;
        let window = windows.single();

        let spawn_position = if spawn_mode.mouse_control {
            if let Some(cursor_pos) = window.cursor_position() {
                let x = cursor_pos.x - window.width() / 2.0;
                let y = window.height() / 2.0 - cursor_pos.y;
                let (x, y) = clamp_bounds(x, y, game_area.width, game_area.height);
                Some((x, y))
            } else {
                None
            }
        } else {
            let x = random_range(-game_area.width / 2.0, game_area.width / 2.0);
            Some((x, game_area.height / 2.0 + 100.0))
        };

        if let Some((x, y)) = spawn_position {
            commands.spawn((
                Sprite {
                    color: Color::srgb(random_f32(), random_f32(), random_f32()),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                Transform::from_xyz(x, y, 0.0),
                Visibility::default(),
                RigidBody::Dynamic,
                Collider::cuboid(15.0, 15.0),
                Velocity::linear(Vec2::new(random_range(-100.0, 100.0), -200.0)),
                FallingCube,
                DelayedDeletion {
                    timer: Timer::from_seconds(20.0, TimerMode::Once),
                },
                StationaryTimer {
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                    is_moving: true,
                },
            ));
            spawn_mode.object_count += 1;
        }
    }
}

fn check_bottom_despawn(
    mut commands: Commands,
    windows: Query<&Window>,
    query: Query<(Entity, &Transform), With<FallingCube>>,
) {
    let window = windows.single();
    let bottom = -window.height() / 2.0;

    for (entity, transform) in query.iter() {
        if transform.translation.y <= bottom {
            commands.entity(entity).despawn();
        }
    }
}

fn check_delayed_deletion(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DelayedDeletion)>,
) {
    for (entity, mut delayed) in &mut query {
        delayed.timer.tick(time.delta());
        if delayed.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_clear_button(
    mut commands: Commands,
    query_cubes: Query<Entity, With<FallingCube>>,
    query_button: Query<&Interaction, (Changed<Interaction>, With<ClearButton>)>,
) {
    for interaction in &query_button {
        if *interaction == Interaction::Pressed {
            for entity in &query_cubes {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn handle_toggle_button(
    mut spawn_mode: ResMut<SpawnMode>,
    query_button: Query<&Interaction, (Changed<Interaction>, With<ToggleButton>)>,
) {
    for interaction in &query_button {
        if *interaction == Interaction::Pressed {
            spawn_mode.mouse_control = !spawn_mode.mouse_control;
        }
    }
}

fn update_screen_dimensions(mut screen_dims: ResMut<ScreenDimensions>, windows: Query<&Window>) {
    let window = windows.single();
    screen_dims.width = window.width();
    screen_dims.height = window.height();
}

fn update_game_area(mut game_area: ResMut<GameArea>, windows: Query<&Window>) {
    let window = windows.single();
    let min_dimension = window.width().min(window.height());
    game_area.width = min_dimension;
    game_area.height = min_dimension;
    game_area.margin = (window.width() - min_dimension) / 2.0;
}

fn check_stationary_objects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Velocity, &mut StationaryTimer)>,
) {
    for (entity, velocity, mut timer) in &mut query {
        let is_moving = velocity.linvel.length_squared() > 1.0;
        
        if !is_moving && timer.is_moving {
            // Object just stopped
            timer.is_moving = false;
            timer.timer.reset();
        } else if is_moving {
            timer.is_moving = true;
        }

        if !timer.is_moving {
            timer.timer.tick(time.delta());
            if timer.timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn move_ground(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut MovingGround)>,
) {
    for (mut transform, mut moving) in query.iter_mut() {
        let new_x = transform.translation.x + moving.speed * moving.direction * time.delta_secs();
        
        if new_x >= moving.bounds {
            moving.direction = -1.0;
        } else if new_x <= -moving.bounds {
            moving.direction = 1.0;
        }
        
        transform.translation.x = new_x.clamp(-moving.bounds, moving.bounds);
    }
}

fn check_respawn_cube(
    mut query: Query<(&mut Transform, &RespawnCube)>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let respawn_height = window.height();
    
    for (mut transform, _) in query.iter_mut() {
        if transform.translation.y <= -window.height() / 2.0 {
            transform.translation.y = respawn_height;
            transform.translation.x = 0.0;
        }
    }
}
