use bevy::prelude::*;
use crate::components::*;
use crate::game_assets::GameAssets;
use crate::states::GameState;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEvent>()
            .add_systems(OnEnter(GameState::Playing), start_music)
            .add_systems(OnExit(GameState::Playing), stop_music)
            .add_systems(Update, play_sound_effects.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::GameOver), start_gameover_music)
            .add_systems(OnExit(GameState::GameOver), stop_music)
            .add_systems(OnEnter(GameState::LevelComplete), start_goal_music)
            .add_systems(OnExit(GameState::LevelComplete), stop_music);
    }
}

#[derive(Component)]
struct MusicMarker;

fn start_music(
    mut commands: Commands,
    assets: Res<GameAssets>,
    data: Res<GameData>,
) {
    let music = if data.level >= 5 {
        assets.mus_castle.clone()
    } else {
        assets.mus_main.clone()
    };
    commands.spawn((
        AudioPlayer(music),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::new(0.5)),
        MusicMarker,
    ));
}

fn stop_music(mut commands: Commands, query: Query<Entity, With<MusicMarker>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

fn start_gameover_music(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        AudioPlayer(assets.mus_gameover.clone()),
        PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::new(0.5)),
        MusicMarker,
    ));
}

fn start_goal_music(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        AudioPlayer(assets.mus_goal.clone()),
        PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::new(0.5)),
        MusicMarker,
    ));
}

fn play_sound_effects(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut events: EventReader<SoundEvent>,
) {
    for event in events.read() {
        let handle = match event {
            SoundEvent::Jump => assets.snd_jump.clone(),
            SoundEvent::Coin => assets.snd_coin.clone(),
            SoundEvent::Stomp => assets.snd_stomp.clone(),
            SoundEvent::Death => assets.snd_death.clone(),
            SoundEvent::Up => assets.snd_1up.clone(),
        };
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::new(0.5)),
        ));
    }
}
