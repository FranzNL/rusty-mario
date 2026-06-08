use bevy::prelude::*;

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    DeathPause, // brief pause before restarting level or going to GameOver
    LevelComplete,
    GameOver,
}

#[derive(Resource, Default)]
pub struct DeathPauseTimer(pub f32); // counts down from ~2.5s
