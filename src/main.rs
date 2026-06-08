use bevy::prelude::*;

mod audio;
mod camera;
mod components;
mod constants;
mod enemies;
mod game_assets;
mod items;
mod level;
mod physics;
mod player;
mod states;
mod fullscreen;
mod touch_controls;
mod ui;

use audio::AudioPlugin;
use camera::CameraPlugin;
use components::{GameData, UiHandles};
use constants::*;
use enemies::EnemiesPlugin;
use game_assets::AssetsPlugin;
use items::ItemsPlugin;
use level::LevelPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use states::{DeathPauseTimer, GameState};
use fullscreen::FullscreenPlugin;
use touch_controls::TouchControlsPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rusty Mario".to_string(),
                        resolution: (WINDOW_W, WINDOW_H).into(),
                        canvas: Some("#bevy".to_string()),
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GameData {
            score: 0,
            highscore: 0,
            lives: 3,
            level: 1,
            time: LEVEL_TIME,
        })
        .insert_resource(UiHandles::default())
        .insert_resource(DeathPauseTimer::default())
        .init_state::<GameState>()
        .add_plugins(AssetsPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemiesPlugin)
        .add_plugins(ItemsPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(FullscreenPlugin)
        .add_plugins(TouchControlsPlugin)
        .add_plugins(UiPlugin)
        .run();
}
