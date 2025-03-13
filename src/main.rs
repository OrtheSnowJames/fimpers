use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, (setup_camera, spawn_ground, spawn_box_collider))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_ground(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.7, 0.2),
                custom_size: Some(Vec2::new(1000.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -300.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(500.0, 25.0),
    ));
}

fn spawn_box_collider(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(25.0, 25.0),
    ));
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
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(rng.gen(), rng.gen(), rng.gen()),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, window.height() / 2.0 + 100.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(15.0, 15.0),
            Velocity::linear(Vec2::new(rng.gen_range(-100.0..100.0), -200.0)),
        ));
    }
}
