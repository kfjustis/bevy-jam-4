use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
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
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("player_pixel_1.png"),
        ..default()
    });
}

fn move_player(
    _keys: Res<Input<KeyCode>>
) {
    return;
}