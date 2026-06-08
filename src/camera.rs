use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::level::LevelMeta;
use crate::states::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Playing), reset_camera)
            .add_systems(
                Update,
                (camera_follow, parallax_background)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(WINDOW_W * 0.5, CAMERA_Y, 100.0),
        MainCamera,
    ));
}

fn reset_camera(mut camera_q: Query<&mut Transform, With<MainCamera>>) {
    if let Ok(mut cam_t) = camera_q.get_single_mut() {
        cam_t.translation.x = WINDOW_W * 0.5;
        cam_t.translation.y = CAMERA_Y;
    }
}

fn camera_follow(
    player_q: Query<&Transform, With<Player>>,
    level_meta: Res<LevelMeta>,
    mut camera_q: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Ok(player_t) = player_q.get_single() else { return; };
    let Ok(mut cam_t) = camera_q.get_single_mut() else { return; };

    let px = player_t.translation.x;
    let cx = cam_t.translation.x;

    // Follow with ±64px dead zone (matching Python camera)
    if px > cx + 64.0 {
        cam_t.translation.x = px - 64.0;
    }
    if px < cx - 64.0 {
        cam_t.translation.x = px + 64.0;
    }

    // Clamp camera so it doesn't show outside the level
    let min_x = WINDOW_W * 0.5;
    let max_x = level_meta.width_px - WINDOW_W * 0.5;
    cam_t.translation.x = cam_t.translation.x.clamp(min_x, max_x.max(min_x));
    // Y stays fixed
    cam_t.translation.y = CAMERA_Y;
}

fn parallax_background(
    camera_q: Query<&Transform, With<MainCamera>>,
    mut bg_q: Query<&mut Transform, (With<Background>, Without<MainCamera>)>,
) {
    let Ok(cam_t) = camera_q.get_single() else { return; };
    let cam_x = cam_t.translation.x;

    // Tiles are placed at i * WINDOW_W + WINDOW_W * 0.5 in spawn.
    // Shift them so bg scrolls at half speed (parallax).
    let bg_offset = cam_x * 0.5;
    let cycle = (bg_offset % WINDOW_W + WINDOW_W) % WINDOW_W;

    for (i, mut bg_t) in bg_q.iter_mut().enumerate() {
        let base = i as f32 * WINDOW_W - WINDOW_W;
        bg_t.translation.x = cam_x - cycle + base;
        bg_t.translation.y = CAMERA_Y;
    }
}
