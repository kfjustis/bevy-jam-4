use bevy::prelude::*;

mod init;
mod game;

fn main() {
    App::new()
        .add_plugins((
            init::InitPlugin,
            game::GamePlugin
        ))
        .run();
}
