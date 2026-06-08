use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::game_assets::GameAssets;
use crate::states::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiHandles>()
            .add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(
                Update,
                update_hud.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                cleanup_ui,
            )
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_ui)
            .add_systems(
                Update,
                main_menu_input.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnEnter(GameState::GameOver), setup_gameover)
            .add_systems(OnExit(GameState::GameOver), cleanup_ui)
            .add_systems(
                Update,
                gameover_input.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnEnter(GameState::DeathPause), setup_death_screen)
            .add_systems(
                Update,
                (death_pause_tick, animate_death_mario)
                    .run_if(in_state(GameState::DeathPause)),
            )
            .add_systems(OnExit(GameState::DeathPause), cleanup_ui)
            .add_systems(OnEnter(GameState::LevelComplete), setup_level_complete)
            .add_systems(
                Update,
                level_complete_tick.run_if(in_state(GameState::LevelComplete)),
            )
            .add_systems(OnExit(GameState::LevelComplete), cleanup_ui);
    }
}

fn setup_hud(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ui_handles: ResMut<UiHandles>,
    data: Res<GameData>,
) {
    // Score text
    let score_e = commands.spawn((
        Text2d::new(format!("Score: {:05}", data.score)),
        TextFont {
            font: assets.font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(WINDOW_W - 80.0, -12.0, 10.0),
        UiEntity,
    )).id();
    ui_handles.score_text = Some(score_e);

    // Lives text
    let lives_e = commands.spawn((
        Text2d::new(format!("x{}", data.lives.max(0))),
        TextFont {
            font: assets.font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(WINDOW_W * 0.5 + 40.0, -24.0, 10.0),
        UiEntity,
    )).id();
    ui_handles.lives_text = Some(lives_e);

    // World text
    let world_e = commands.spawn((
        Text2d::new(format!("World 1-{}", data.level)),
        TextFont {
            font: assets.font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(WINDOW_W * 0.5 - 80.0, -12.0, 10.0),
        UiEntity,
    )).id();
    ui_handles.world_text = Some(world_e);

    // Time text
    let time_e = commands.spawn((
        Text2d::new(format!("Time: {}", data.time as i32)),
        TextFont {
            font: assets.font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(WINDOW_W - 60.0, -60.0, 10.0),
        UiEntity,
    )).id();
    ui_handles.time_text = Some(time_e);

    // Mario icon (lives indicator)
    commands.spawn((
        Sprite {
            image: assets.mario_life_icon.clone(),
            ..default()
        },
        Transform::from_xyz(WINDOW_W * 0.5, -16.0, 10.0).with_scale(Vec3::splat(SPRITE_SCALE)),
        UiEntity,
    ));
}

fn update_hud(
    data: Res<GameData>,
    camera_q: Query<&Transform, With<MainCamera>>,
    ui_handles: Res<UiHandles>,
    mut text_q: Query<(&mut Text2d, &mut Transform), (With<UiEntity>, Without<MainCamera>)>,
) {
    let Ok(cam_t) = camera_q.get_single() else { return; };
    let cam_x = cam_t.translation.x;
    // HUD follows camera (since we're using world-space 2D text)
    let hud_left = cam_x - WINDOW_W * 0.5;
    let hud_top = CAMERA_Y + WINDOW_H * 0.5;

    // Update score
    if let Some(e) = ui_handles.score_text {
        if let Ok((mut text, mut t)) = text_q.get_mut(e) {
            text.0 = format!("Score: {:05}", data.score);
            t.translation.x = hud_left + WINDOW_W - 80.0;
            t.translation.y = hud_top - 12.0;
        }
    }
    // Update lives
    if let Some(e) = ui_handles.lives_text {
        if let Ok((mut text, mut t)) = text_q.get_mut(e) {
            text.0 = format!("x{}", data.lives.max(0));
            t.translation.x = hud_left + WINDOW_W * 0.5 + 40.0;
            t.translation.y = hud_top - 24.0;
        }
    }
    // Update world
    if let Some(e) = ui_handles.world_text {
        if let Ok((mut text, mut t)) = text_q.get_mut(e) {
            text.0 = format!("World 1-{}", data.level);
            t.translation.x = hud_left + WINDOW_W * 0.5 - 80.0;
            t.translation.y = hud_top - 12.0;
        }
    }
    // Update time
    if let Some(e) = ui_handles.time_text {
        if let Ok((mut text, mut t)) = text_q.get_mut(e) {
            text.0 = format!("Time: {}", data.time as i32);
            t.translation.x = hud_left + WINDOW_W - 80.0;
            t.translation.y = hud_top - 48.0;
        }
    }
}

fn cleanup_ui(mut commands: Commands, q: Query<Entity, With<UiEntity>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

// ── Main Menu ────────────────────────────────────────────────────────
fn setup_main_menu(mut commands: Commands, assets: Res<GameAssets>) {
    let cx = WINDOW_W * 0.5;
    let cy = CAMERA_Y;
    commands.spawn((
        Text2d::new("RUSTY MARIO"),
        TextFont { font: assets.font.clone(), font_size: 48.0, ..default() },
        TextColor(Color::srgb(1.0, 0.8, 0.1)),
        Transform::from_xyz(cx, cy + 80.0, 10.0),
        UiEntity,
    ));
    commands.spawn((
        Text2d::new("Press Z to Start"),
        TextFont { font: assets.font.clone(), font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(cx, cy, 10.0),
        UiEntity,
    ));
    commands.spawn((
        Text2d::new("Arrow keys: Move   Z: Jump   X: Shoot"),
        TextFont { font: assets.font.clone(), font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Transform::from_xyz(cx, cy - 60.0, 10.0),
        UiEntity,
    ));
}

fn main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
    mut data: ResMut<GameData>,
) {
    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Enter) {
        data.score = 0;
        data.highscore = 0;
        data.lives = 3;
        data.level = 1;
        data.time = LEVEL_TIME;
        next.set(GameState::Playing);
    }
}

// ── Game Over ────────────────────────────────────────────────────────
fn setup_gameover(mut commands: Commands, assets: Res<GameAssets>, data: Res<GameData>) {
    let cx = WINDOW_W * 0.5;
    let cy = CAMERA_Y;
    commands.spawn((
        Text2d::new("GAME OVER"),
        TextFont { font: assets.font.clone(), font_size: 48.0, ..default() },
        TextColor(Color::srgb(1.0, 0.2, 0.2)),
        Transform::from_xyz(cx, cy + 80.0, 10.0),
        UiEntity,
    ));
    commands.spawn((
        Text2d::new(format!("Score: {:05}", data.score)),
        TextFont { font: assets.font.clone(), font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(cx, cy + 20.0, 10.0),
        UiEntity,
    ));
    commands.spawn((
        Text2d::new("Press Z to play again"),
        TextFont { font: assets.font.clone(), font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        Transform::from_xyz(cx, cy - 60.0, 10.0),
        UiEntity,
    ));
}

fn gameover_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
    mut data: ResMut<GameData>,
) {
    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Enter) {
        data.score = 0;
        data.lives = 3;
        data.level = 1;
        data.time = LEVEL_TIME;
        next.set(GameState::Playing);
    }
}

// ── Death Pause ──────────────────────────────────────────────────────
fn setup_death_screen(
    mut commands: Commands,
    assets: Res<GameAssets>,
    data: Res<GameData>,
    camera_q: Query<&Transform, With<MainCamera>>,
) {
    let cx = camera_q.get_single().map(|t| t.translation.x).unwrap_or(WINDOW_W * 0.5);
    let cy = CAMERA_Y;

    // Bouncing mario die sprite
    commands.spawn((
        Sprite {
            image: assets.mario_die.clone(),
            ..default()
        },
        Transform::from_xyz(cx, cy, 10.0).with_scale(Vec3::splat(SPRITE_SCALE)),
        crate::components::DeathMario { vel_y: 380.0 },
        UiEntity,
    ));

    let title = if data.lives > 0 { "YOU DIED" } else { "GAME OVER" };
    commands.spawn((
        Text2d::new(title),
        TextFont { font: assets.font.clone(), font_size: 36.0, ..default() },
        TextColor(Color::srgb(1.0, 0.4, 0.4)),
        Transform::from_xyz(cx, cy + 100.0, 11.0),
        UiEntity,
    ));
    commands.spawn((
        Text2d::new(format!("Lives: {}", data.lives.max(0))),
        TextFont { font: assets.font.clone(), font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(cx, cy + 60.0, 11.0),
        UiEntity,
    ));
}

fn animate_death_mario(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut crate::components::DeathMario)>,
) {
    let dt = time.delta_secs();
    for (mut t, mut dm) in q.iter_mut() {
        dm.vel_y -= 900.0 * dt;
        t.translation.y += dm.vel_y * dt;
    }
}

fn death_pause_tick(
    time: Res<Time>,
    mut timer: ResMut<crate::states::DeathPauseTimer>,
    mut data: ResMut<GameData>,
    mut next: ResMut<NextState<GameState>>,
) {
    timer.0 -= time.delta_secs();
    if timer.0 <= 0.0 {
        if data.lives <= 0 {
            next.set(GameState::GameOver);
        } else {
            data.time = LEVEL_TIME;
            next.set(GameState::Playing);
        }
    }
}

// ── Level Complete ───────────────────────────────────────────────────
fn setup_level_complete(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut data: ResMut<GameData>,
) {
    data.level += 1;
    if data.level > 4 {
        // Wrap back to level 1 after completing all 4
        data.level = 1;
    }
    data.time = LEVEL_TIME;

    let cx = WINDOW_W * 0.5;
    let cy = CAMERA_Y;
    commands.spawn((
        Text2d::new("LEVEL CLEAR!"),
        TextFont { font: assets.font.clone(), font_size: 36.0, ..default() },
        TextColor(Color::srgb(0.2, 1.0, 0.4)),
        Transform::from_xyz(cx, cy + 40.0, 10.0),
        UiEntity,
    ));
    commands.spawn((
        Text2d::new(format!("Next: World 1-{}", data.level)),
        TextFont { font: assets.font.clone(), font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(cx, cy - 20.0, 10.0),
        UiEntity,
    ));
}

fn level_complete_tick(
    time: Res<Time>,
    mut timer: Local<f32>,
    mut next: ResMut<NextState<GameState>>,
) {
    *timer += time.delta_secs();
    if *timer >= 3.0 {
        *timer = 0.0;
        next.set(GameState::Playing);
    }
}
