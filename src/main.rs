use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;

mod init;
mod game;
mod progressbar;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            init::InitPlugin,
            game::GamePlugin,
            progressbar::ProgressBarPlugin
        ))
        .run();
}
