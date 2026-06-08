use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::game_assets::GameAssets;
use crate::states::GameState;
use crate::level::LevelMeta;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                coin_animation,
                collect_coins,
                collect_mushrooms,
                collect_level_exit,
                check_right_edge,
                effect_animation,
                time_countdown,
                bump_platform_q,
                pop_coin_update,
                plant_update,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn coin_animation(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut Coin)>,
    assets: Res<GameAssets>,
) {
    let dt = time.delta_secs();
    for (mut sprite, mut coin) in query.iter_mut() {
        coin.frame += dt * 60.0;
        let idx = (coin.frame / 6.0) as usize % 4;
        sprite.image = assets.coin[idx].clone();
    }
}

fn collect_coins(
    mut commands: Commands,
    player_q: Query<(&Transform, &BoxCollider), With<Player>>,
    coin_q: Query<(Entity, &Transform, &BoxCollider), With<Coin>>,
    mut data: ResMut<GameData>,
    mut sound_events: EventWriter<SoundEvent>,
    assets: Res<GameAssets>,
) {
    let Ok((p_t, p_c)) = player_q.get_single() else { return; };
    let p_pos = p_t.translation.truncate();
    let p_min = p_pos - p_c.size * 0.5;
    let p_max = p_pos + p_c.size * 0.5;

    for (entity, c_t, c_c) in coin_q.iter() {
        let c_pos = c_t.translation.truncate();
        let c_min = c_pos - c_c.size * 0.5;
        let c_max = c_pos + c_c.size * 0.5;

        if p_min.x < c_max.x && p_max.x > c_min.x && p_min.y < c_max.y && p_max.y > c_min.y {
            data.score += SCORE_COIN;
            sound_events.send(SoundEvent::Coin);
            // Spawn collect effect
            commands.spawn((
                Sprite { image: assets.exp2[0].clone(), ..default() },
                Transform::from_xyz(c_pos.x, c_pos.y, 5.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                EffectAnimation {
                    timer: 0.0,
                    frames: 3,
                    frame_duration: 0.06,
                    current_frame: 0,
                    sprites: vec![
                        assets.exp2[0].clone(),
                        assets.exp2[1].clone(),
                        assets.exp2[2].clone(),
                    ],
                },
                LevelEntity,
            ));
            commands.entity(entity).despawn();
        }
    }
}

fn collect_mushrooms(
    mut commands: Commands,
    mut player_q: Query<(&Transform, &BoxCollider, &mut Player), With<Player>>,
    mush_q: Query<(Entity, &Transform, &BoxCollider), With<MushroomGreen>>,
    mut data: ResMut<GameData>,
    mut sound_events: EventWriter<SoundEvent>,
    assets: Res<GameAssets>,
) {
    let Ok((p_t, p_c, mut player)) = player_q.get_single_mut() else { return; };
    let p_pos = p_t.translation.truncate();
    let p_min = p_pos - p_c.size * 0.5;
    let p_max = p_pos + p_c.size * 0.5;

    for (entity, m_t, m_c) in mush_q.iter() {
        let m_pos = m_t.translation.truncate();
        let m_min = m_pos - m_c.size * 0.5;
        let m_max = m_pos + m_c.size * 0.5;

        if p_min.x < m_max.x && p_max.x > m_min.x && p_min.y < m_max.y && p_max.y > m_min.y {
            data.score += SCORE_MUSHROOM;
            if player.hp >= 2 {
                data.lives += 1; // already big — give a life instead
            } else {
                player.hp = 2; // power up to big Mario
            }
            sound_events.send(SoundEvent::Up);
            commands.spawn((
                Sprite { image: assets.exp2[0].clone(), ..default() },
                Transform::from_xyz(m_pos.x, m_pos.y, 5.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                EffectAnimation {
                    timer: 0.0,
                    frames: 3,
                    frame_duration: 0.08,
                    current_frame: 0,
                    sprites: vec![
                        assets.exp2[0].clone(),
                        assets.exp2[1].clone(),
                        assets.exp2[2].clone(),
                    ],
                },
                LevelEntity,
            ));
            commands.entity(entity).despawn();
        }
    }
}

fn collect_level_exit(
    player_q: Query<(&Transform, &BoxCollider), With<Player>>,
    exit_q: Query<(&Transform, &BoxCollider), With<LevelExit>>,
    mut data: ResMut<GameData>,
    mut next: ResMut<NextState<GameState>>,
) {
    let Ok((p_t, p_c)) = player_q.get_single() else { return; };
    let p_pos = p_t.translation.truncate();
    let p_min = p_pos - p_c.size * 0.5;
    let p_max = p_pos + p_c.size * 0.5;

    for (e_t, e_c) in exit_q.iter() {
        let e_pos = e_t.translation.truncate();
        let e_min = e_pos - e_c.size * 0.5;
        let e_max = e_pos + e_c.size * 0.5;

        if p_min.x < e_max.x && p_max.x > e_min.x && p_min.y < e_max.y && p_max.y > e_min.y {
            data.score += SCORE_BOMB;
            next.set(GameState::LevelComplete);
            return;
        }
    }
}

fn check_right_edge(
    player_q: Query<&Transform, With<Player>>,
    level_meta: Res<LevelMeta>,
    exit_q: Query<(), With<LevelExit>>,
    mut data: ResMut<GameData>,
    mut next: ResMut<NextState<GameState>>,
) {
    let Ok(transform) = player_q.get_single() else { return; };
    // If player reaches right edge and there's no bomb exit, auto-advance level
    if transform.translation.x >= level_meta.width_px && exit_q.is_empty() {
        data.level += 1;
        next.set(GameState::LevelComplete);
    }
}

fn effect_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut EffectAnimation)>,
) {
    let dt = time.delta_secs();
    for (entity, mut sprite, mut anim) in query.iter_mut() {
        anim.timer += dt;
        if anim.timer >= anim.frame_duration {
            anim.timer = 0.0;
            anim.current_frame += 1;
            if anim.current_frame >= anim.frames {
                commands.entity(entity).despawn();
                continue;
            }
            sprite.image = anim.sprites[anim.current_frame].clone();
        }
    }
}

fn plant_update(
    time: Res<Time>,
    mut rose_q: Query<(&mut Transform, &mut PipeDweller, &mut Sprite), (With<RosePlant>, Without<FlowerPlant>)>,
    mut flower_q: Query<(&mut Transform, &mut PipeDweller, &mut Sprite), (With<FlowerPlant>, Without<RosePlant>)>,
    assets: Res<GameAssets>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let frame = (t * 3.0) as usize % 2;

    for (mut tf, mut pd, mut sp) in rose_q.iter_mut() {
        sp.image = assets.rose[frame].clone();
        tick_plant(&mut tf, &mut pd, &mut sp, dt);
    }
    for (mut tf, mut pd, mut sp) in flower_q.iter_mut() {
        sp.image = assets.flower[frame].clone();
        tick_plant(&mut tf, &mut pd, &mut sp, dt);
    }
}

fn tick_plant(tf: &mut Transform, pd: &mut PipeDweller, sp: &mut Sprite, dt: f32) {
    match pd.state {
        PipeDwellerState::Hidden => {
            tf.translation.y = pd.hidden_y;
            sp.color = Color::srgba(1., 1., 1., 0.);
            pd.timer -= dt;
            if pd.timer <= 0.0 {
                pd.state = PipeDwellerState::Emerging;
            }
        }
        PipeDwellerState::Emerging => {
            sp.color = Color::WHITE;
            tf.translation.y += PIPE_EMERGE_SPEED * dt;
            if tf.translation.y >= pd.emerged_y {
                tf.translation.y = pd.emerged_y;
                pd.state = PipeDwellerState::Out;
                pd.timer = PIPE_OUT_DURATION;
            }
        }
        PipeDwellerState::Out => {
            sp.color = Color::WHITE;
            pd.timer -= dt;
            if pd.timer <= 0.0 {
                pd.state = PipeDwellerState::Retreating;
            }
        }
        PipeDwellerState::Retreating => {
            sp.color = Color::WHITE;
            tf.translation.y -= PIPE_EMERGE_SPEED * dt;
            if tf.translation.y <= pd.hidden_y {
                tf.translation.y = pd.hidden_y;
                pd.state = PipeDwellerState::Hidden;
                pd.timer = PIPE_HIDDEN_DURATION;
            }
        }
    }
}

fn bump_platform_q(
    mut commands: Commands,
    mut bump_events: EventReader<BumpEvent>,
    mut pq_query: Query<(&Transform, &mut PlatformQ)>,
    assets: Res<GameAssets>,
) {
    for BumpEvent(entity) in bump_events.read() {
        let Ok((transform, mut pq)) = pq_query.get_mut(*entity) else { continue; };
        if pq.used { continue; }
        pq.used = true;
        pq.bump_timer = 0.3;

        // Spawn a pop coin just above the box
        let spawn_y = transform.translation.y + TILE * SPRITE_SCALE;
        commands.spawn((
            Sprite { image: assets.coin[0].clone(), ..default() },
            Transform::from_xyz(transform.translation.x, spawn_y, 4.0)
                .with_scale(Vec3::splat(SPRITE_SCALE)),
            PopCoin { vel_y: 300.0, timer: 0.55, frame: 0.0 },
            LevelEntity,
        ));
    }
}

fn pop_coin_update(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut PopCoin)>,
    mut data: ResMut<GameData>,
    mut sound_events: EventWriter<SoundEvent>,
    assets: Res<GameAssets>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut sprite, mut pop) in query.iter_mut() {
        pop.timer -= dt;
        pop.frame += dt * 60.0;
        pop.vel_y -= 900.0 * dt; // gravity pulls it back
        transform.translation.y += pop.vel_y * dt;

        let idx = (pop.frame / 6.0) as usize % 4;
        sprite.image = assets.coin[idx].clone();

        if pop.timer <= 0.0 {
            data.score += SCORE_COIN;
            sound_events.send(SoundEvent::Coin);
            commands.entity(entity).despawn();
        }
    }
}

fn time_countdown(
    time: Res<Time>,
    mut data: ResMut<GameData>,
    player_q: Query<&Player>,
    mut next: ResMut<NextState<GameState>>,
    mut death_timer: ResMut<crate::states::DeathPauseTimer>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    if player_q.get_single().is_ok() {
        data.time -= time.delta_secs() * 1.0; // 1 unit per real second (Python: ~0.06/frame ≈ 3.6/s but let's slow it down)
        if data.time <= 0.0 {
            data.time = 0.0;
            // Time ran out: kill player
            sound_events.send(SoundEvent::Death);
            data.lives -= 1;
            data.score = 0;
            death_timer.0 = 2.5;
            next.set(GameState::DeathPause);
        }
    }
}
