use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::render::settings::{Backends, WgpuSettings, RenderCreation};
use bevy::render::RenderPlugin;
use bevy::window::{PresentMode, WindowTheme};

pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App)
    {
        app.insert_resource(AssetMetaCheck::Never);

        let win_title = "Bevy Jam 4 | kftoons";
        app.add_plugins(DefaultPlugins.set(
            RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    #[cfg(target_os = "windows")]
                    backends: Some(Backends::VULKAN),
                    #[cfg(target_os = "linux")]
                    backends: Some(Backends::GL),
                    ..default()
                }),
                ..default()
            }).set(
                WindowPlugin {
                    primary_window: Some(Window {
                        title: win_title.into(),
                        resolution: (640., 480.).into(),
                        present_mode: PresentMode::AutoVsync,
                        window_theme: Some(WindowTheme::Dark),
                        ..default()
                    }),
                    ..default()
                }).set(ImagePlugin::default_nearest()));
    }
}
