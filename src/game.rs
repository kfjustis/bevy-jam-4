use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::render::camera::ScalingMode;
use bevy_xpbd_2d::{math::*, prelude::*};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            WorldInspectorPlugin::new()
        ));
        app.register_type::<Speed>();
        app.add_systems(Startup, (setup_game).chain());
        app.add_systems(Update, move_player);
        app.insert_resource(Gravity(Vector::NEG_Y * 100.0 * 10.0));
    }
}

#[derive(Component)]
struct Player;

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Speed(f32);

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.0,
            //scaling_mode: ScalingMode::Fixed { width: 320.0, height: 240.0 },
            //scaling_mode: ScalingMode::AutoMin { min_width: 320.0, min_height: 240.0 },
            scaling_mode: ScalingMode::FixedVertical(240.0),
            near: -1000.0,
            far: 1000.0,
            ..default()
        },
        ..default()
    });

    // Spawn the player.
    commands.spawn((
        Name::new("PlayerEntity"),
        Player,
        SpriteBundle {
            texture: asset_server.load("player_pixel_1.png"),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::capsule(32.0, 16.0),
        LockedAxes::new()
            .lock_rotation(),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        Mass(1.0),
        Speed(32.0)
    ));

    // Spawn the ground.
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.63,0.46,0.06),
                custom_size: Some(Vec2::new(320.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -104.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(320.0, 32.0)
    ));
}

fn move_player(
    keys: Res<Input<KeyCode>>,
    mut players: Query<(&mut LinearVelocity, &Speed), With<Player>>,
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