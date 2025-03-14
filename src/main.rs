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

#[derive(Component)]
struct FallingCube;

#[derive(Component)]
struct ClearButton;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<ScreenDimensions>()
        .add_systems(Startup, (setup_camera, spawn_ground, spawn_box_collider, spawn_clear_button))
        .add_systems(Update, (
            update_screen_dimensions,
            throw_cubes,
            check_bottom_despawn,
            handle_clear_button,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_ground(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.7, 0.2),
            custom_size: Some(Vec2::new(1000.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -300.0, 0.0),
        Visibility::default(),
        RigidBody::Fixed,
        Collider::cuboid(500.0, 25.0),
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
    ));
}

const NORMAL_BUTTON: Color = Color::rgb(0.2, 0.2, 0.2);

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

fn random_f32() -> f32 {
    let mut buf = [0u8; 4];
    getrandom(&mut buf).unwrap();
    let random_bits = u32::from_le_bytes(buf);
    (random_bits as f32) / (u32::MAX as f32)
}

fn random_range(min: f32, max: f32) -> f32 {
    min + random_f32() * (max - min)
}

fn throw_cubes(
    mut commands: Commands,
    time: Res<Time>,
    mut last_spawn: Local<f32>,
    windows: Query<&Window>,
) {
    let now = time.elapsed_secs();
    if now - *last_spawn > 0.5 {
        *last_spawn = now;

        let window = windows.single();
        let x = random_range(-window.width() / 2.0, window.width() / 2.0);

        commands.spawn((
            Sprite {
                color: Color::srgb(random_f32(), random_f32(), random_f32()),
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(x, window.height() / 2.0 + 100.0, 0.0),
            Visibility::default(),
            RigidBody::Dynamic,
            Collider::cuboid(15.0, 15.0),
            Velocity::linear(Vec2::new(random_range(-100.0, 100.0), -200.0)),
            FallingCube,
        ));
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

fn update_screen_dimensions(mut screen_dims: ResMut<ScreenDimensions>, windows: Query<&Window>) {
    let window = windows.single();
    screen_dims.width = window.width();
    screen_dims.height = window.height();
}
