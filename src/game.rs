use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_xpbd_2d::{math::*, prelude::*, parry::query::DefaultQueryDispatcher};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ));
        app.add_systems(Startup, setup_game);
        app.add_systems(Update, move_player);
    }
}

#[derive(Component)]
struct Player;

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn(Camera2dBundle {
        //projection: OrthographicProjection {
        //    //scaling_mode: ScalingMode::Fixed { width: 320.0, height: 240.0 },
        //    ..default()
        //},
        ..default()
    });

    // Spawn the player.
    commands.spawn((
        Player,
        SpriteBundle {
            texture: asset_server.load("player_pixel_1.png"),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::capsule(32.0, 16.0)
    ));

    // Spawn the ground.
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.63,0.46,0.06),
                custom_size: Some(Vec2::new(640.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -160.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(640.0, 32.0)
    ));
}

fn move_player(
    _keys: Res<Input<KeyCode>>
) {
    return;
}