use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::render::camera::ScalingMode;
use bevy_scroller::{
    Scroller, ScrollerBundle, ScrollerDirection, ScrollerPlugin, ScrollerSize, SingleSpriteGenerator
};
use bevy_tweening::{lens::*, *};
use bevy_xpbd_2d::{math::*, prelude::*};
use rand::Rng;
use std::collections::VecDeque;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            ScrollerPlugin,
            TweeningPlugin,
            WorldInspectorPlugin::new()
        ));
        app.register_type::<Speed>();
        app.add_systems(Startup, (setup_game, setup_snow).chain());
        app.add_systems(Update, (
                anim_player,
                move_player,
                spawn_snow,
                move_snow,
                collide_snow,
                remove_snow
            ).chain());
        app.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)));
        app.insert_resource(Gravity(Vector::NEG_Y * 100.0 * 10.0));
    }
}

#[derive(Resource)]
pub struct ScrollerImages(Vec<Handle<Image>>);

#[derive(Component)]
struct PlayerCapsule;

#[derive(Component)]
struct PlayerSprite;

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Speed(f32);

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct SnowTile;

#[derive(Resource)]
struct SnowConfig {
    // How often the snow should spawn.
    timer: Timer,
}

// Define the collision layers
#[derive(PhysicsLayer)]
enum Layer {
    Player,
    Wall,
    Ground,
    Snow
}

#[derive(Component)]
struct ToDelete;

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.0,
            scaling_mode: ScalingMode::Fixed { width: 320.0, height: 240.0 },
            near: -1000.0,
            far: 1000.0,
            ..default()
        },
        ..default()
    });

    // Spawn the bg layers.
    let bg_layers = (0..=4)
        .map(|i| format!("parallax/{i}.png"))
        .collect::<VecDeque<String>>();
    let bg_layer_handles = bg_layers
        .iter()
        .map(|image_path| asset_server.load(image_path))
        .collect::<Vec<Handle<Image>>>();
    commands.insert_resource(ScrollerImages(bg_layer_handles));
    let bg_sizes = [
        Vec2::new(320.0, 240.0),
        Vec2::new(320.0, 240.0),
        Vec2::new(320.0, 240.0),
        Vec2::new(320.0, 240.0),
        Vec2::new(320.0, 240.0),
    ];
    let scroller_speed_min = 0.2;
    let scroller_speed_step = 0.2;
    bg_sizes.into_iter().enumerate().for_each(|(i, size)| {
        commands.spawn((
            ScrollerSize {
                size: Vec2::new(320.0, 240.0)
            },
            ScrollerBundle {
                scroller: Scroller {
                    speed: scroller_speed_min + i as f32 * scroller_speed_step,
                    direction: ScrollerDirection::Backward,
                    render_layer: Some(1),
                    ..default()
                },
                generator: SingleSpriteGenerator {
                    path: format!("parallax/{i}.png"),
                    size,
                },
                spatial: SpatialBundle::from_transform(Transform::from_translation(
                    Vec3::new(0.0, 0.0, 1.0 + i as f32)
                )),
                ..default()
            },
        ));
    });

    // Spawn the player with its physics, sprite, and tween animations.
    // The sprite is a child of the capsule/SpatialBundle so it can
    // rotate independently.
    commands.spawn((
        Name::new("PlayerEntity"),
        PlayerCapsule,
        SpatialBundle {
            visibility: Visibility::Inherited,
            transform: Transform::from_xyz(0.0, 0.0, 100.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::capsule(32.0, 16.0),
        CollisionLayers::new([Layer::Player],
            [Layer::Ground, Layer::Wall, Layer::Snow]),
        LockedAxes::new().lock_rotation(),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        Mass(1.0),
        Speed(32.0)
    ))
    .with_children(|parent| {
        let base_duration_ms: u64 = 500;
        let tween = Tween::new(
            EaseFunction::ElasticInOut,
            std::time::Duration::from_millis(base_duration_ms),
            TransformRotationLens {
                start: Quat::from_axis_angle(Vec3::Z, -std::f32::consts::PI / 9.),
                end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 9.),
            }
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::Repeat);

        parent.spawn((
            PlayerSprite,
            SpriteBundle {
                texture: asset_server.load("player_pixel_1.png"),
                ..default()
            },
            Animator::new(tween)));
    });

    // Spawn the ground.
    commands.spawn((
        Name::new("Ground"),
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.95, 0.95, 1.0),
                custom_size: Some(Vec2::new(640.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -104.0, 200.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(640.0, 32.0),
        CollisionLayers::new([Layer::Ground],
            [Layer::Player, Layer::Snow]),
    ));

    // Spawn the walls.
    commands.spawn((
        Name::new("WallEntityL"),
        Wall,
        SpatialBundle {
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(-192.0, -56.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(64.0, 64.0),
        CollisionLayers::new([Layer::Wall],
            [Layer::Player]),
    ));
    commands.spawn((
        Name::new("WallEntityR"),
        Wall,
        SpatialBundle {
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(192.0, -56.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(64.0, 64.0),
        CollisionLayers::new([Layer::Wall],
            [Layer::Player]),
    ));
}

fn anim_player(
    keys: Res<Input<KeyCode>>,
    mut animators: Query<&mut Animator<Transform>, With<PlayerSprite>>
) {
    let hold_action = keys.any_pressed([KeyCode::Space]);

    for mut animator in animators.iter_mut() {
        let base_duration_ms: u64 = 500;
        let norm_speed = animator.speed() < 1.5f32;
        let fast_speed = animator.speed() > 1.5f32;

        if norm_speed && hold_action {
            let tween = Tween::new(
                EaseFunction::ElasticInOut,
                std::time::Duration::from_millis(base_duration_ms),
                TransformRotationLens {
                    // These angles lean the sprite forward. (Towards screen right.)
                    start: Quat::from_axis_angle(Vec3::Z, -std::f32::consts::PI / 4.),
                    end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 12.),
                }
            )
            .with_repeat_count(RepeatCount::Infinite)
            .with_repeat_strategy(RepeatStrategy::Repeat);
            animator.set_tweenable(tween);
            animator.set_speed(2.0);
        } else if fast_speed && !hold_action{
            let tween = Tween::new(
                EaseFunction::ElasticInOut,
                std::time::Duration::from_millis(base_duration_ms),
                TransformRotationLens {
                    start: Quat::from_axis_angle(Vec3::Z, -std::f32::consts::PI / 9.),
                    end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 9.),
                }
            )
            .with_repeat_count(RepeatCount::Infinite)
            .with_repeat_strategy(RepeatStrategy::Repeat);
            animator.set_tweenable(tween);
            animator.set_speed(1.0);
        }
    }
}

fn move_player(
    keys: Res<Input<KeyCode>>,
    mut players: Query<(&mut LinearVelocity, &Speed), With<PlayerCapsule>>
) {
    for (mut linear_vel, player_speed) in &mut players {
        // Only move left and right.
        let mut direction = Vec2::ZERO;
        let left = keys.any_pressed([KeyCode::A, KeyCode::Left]);
        let right = keys.any_pressed([KeyCode::D, KeyCode::Right]);
        if left {
            direction += Vec2::new(-1.0, 0.0);
        }
        if right {
            direction += Vec2::new(1.0, 0.0);
        }
        direction.y = 0.0;

        // Set the velocity.
        linear_vel.x += direction.normalize_or_zero().x * player_speed.0;

        // Apply friction.
        linear_vel.x *= 0.8;
    }
}

fn setup_snow(
    mut commands: Commands
) {
    commands.insert_resource(
        SnowConfig {
            // Create the repeating timer.
            timer: Timer::new(std::time::Duration::from_millis(500), TimerMode::Repeating)
        }
    );
}

fn spawn_snow(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut config: ResMut<SnowConfig>
) {
    // Tick the snow timer.
    config.timer.tick(time.delta());

    if config.timer.finished() {
        // Pick a random snow sprite each time.
        let mut rng = rand::thread_rng();
        let sprites = vec!["snow_1.png", "snow_2.png"];
        let sprite_idx: usize = rng.gen_range(0..sprites.len());
        let path = sprites[sprite_idx];
        // Spawn the snow sprite with its physics components.
        commands.spawn((
            Name::new("SnowTile"),
            SnowTile,
            SpriteBundle {
                texture: asset_server.load(path),
                // Just above the player in z-order.
                transform: Transform::from_xyz(192.0, 0.0, 300.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(32.0, 32.0),
            CollisionLayers::new([Layer::Snow],
                [Layer::Player, Layer::Ground]),
            //LockedAxes::new().lock_rotation(),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            Mass(1.0),
            Speed(24.0)
        ));
    }
}

fn move_snow(
    mut snow: Query<&mut LinearVelocity, With<SnowTile>>
) {
    let snow_speed = 1.0;
    for mut linear_vel in &mut snow {
        linear_vel.x += Vec2::new(-1.0, 0.0).normalize_or_zero().x * snow_speed;
    }
}

fn collide_snow(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    player: Query<Entity, With<PlayerCapsule>>,
    mut collisions: Query<(Entity, &mut LinearVelocity, &CollidingEntities), With<SnowTile>>
) {
    let force = 160.0;
    let hold_action = keys.any_pressed([KeyCode::Space]);
    for (entity, mut linear_vel, colliding_entities) in &mut collisions {
        if hold_action && colliding_entities.contains(&player.single())
        {
            linear_vel.x += Vec2::new(1.0, 0.0).x * force;
            linear_vel.y += Vec2::new(0.0, 2.0).y * force;

            // Add the toDelete component.
            commands.entity(entity).insert(ToDelete);
        }
    }
}

fn remove_snow(
    mut commands: Commands,
    snow: Query<(Entity, &Transform), (With<SnowTile>, With<ToDelete>)>
) {
    for (entity, transform) in &snow {
        if transform.translation.x < -130.0 || transform.translation.x > 250.0 ||
            transform.translation.y < -100.0 || transform.translation.y > 150.0
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}