use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::game_assets::GameAssets;
use crate::states::GameState;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        // Critical path chained: stomp always runs before collision check,
        // cleanup_dead_enemies always runs last to avoid double-despawn.
        app.add_systems(
            Update,
            (
                update_enemies,
                stomp_detection,
                enemy_player_collision,
                cleanup_dead_enemies,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                update_projectiles,
                hazard_collision,
                moving_platform_update,
                platform_q_update,
                spring_animation,
                pipe_dweller_update,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn update_enemies(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_q: Query<
        (&mut Enemy, &mut Sprite, &mut Velocity, &mut AnimationState, &Transform, &BoxCollider, Option<&PipeDweller>),
        Without<Player>,
    >,
    static_q: Query<(&Transform, &BoxCollider), (With<Solid>, Without<Enemy>)>,
    player_q: Query<&Transform, With<Player>>,
    assets: Res<GameAssets>,
    camera_q: Query<&Transform, With<MainCamera>>,
) {
    let dt = time.delta_secs();
    let player_pos = player_q.get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);
    let camera_x = camera_q.get_single().map(|t| t.translation.x).unwrap_or(320.0);

    let statics: Vec<(Vec2, Vec2)> = static_q
        .iter()
        .map(|(t, c)| {
            let half = c.size * 0.5;
            let pos = t.translation.truncate();
            (pos - half, pos + half)
        })
        .collect();

    for (mut enemy, mut sprite, mut vel, mut anim, transform, coll, pipe) in enemy_q.iter_mut() {
        anim.frame += dt * 60.0;
        enemy.hit_timer -= dt;
        enemy.shoot_timer -= dt;

        let pos = transform.translation;
        let visible = (pos.x - camera_x).abs() < WINDOW_W;

        match enemy.kind {
            EnemyKind::Monster | EnemyKind::MonsterRed | EnemyKind::Slub => {
                if visible {
                    let imgs = match enemy.kind {
                        EnemyKind::Slub => &assets.slub[..2],
                        EnemyKind::MonsterRed => &assets.monster_red[..2],
                        _ => &assets.monster[..2],
                    };
                    let frame = (anim.frame / 8.0) as usize % 2;
                    sprite.image = imgs[frame].clone();
                    sprite.flip_x = enemy.speed > 0.0;

                    let foot_x = if enemy.speed > 0.0 {
                        pos.x + coll.size.x * 0.5 + 1.0
                    } else {
                        pos.x - coll.size.x * 0.5 - 1.0
                    };
                    let foot_y = pos.y - coll.size.y * 0.5 - 2.0;
                    let has_ground = statics.iter().any(|(min, max)| {
                        foot_x >= min.x && foot_x <= max.x && foot_y >= min.y && foot_y <= max.y
                    });

                    if vel.x.abs() < 1.0 || !has_ground {
                        enemy.speed = -enemy.speed;
                        vel.x = enemy.speed;
                    } else {
                        vel.x = enemy.speed;
                    }
                }
            }

            EnemyKind::Squidge => {
                if visible {
                    let frame = (anim.frame / 12.0) as usize % 2;
                    sprite.image = assets.squidge[frame].clone();
                    vel.x = 0.0;
                    if enemy.shoot_timer <= 0.0 {
                        let dx = player_pos.x - pos.x;
                        let dy = player_pos.y - pos.y;
                        let len = (dx * dx + dy * dy).sqrt().max(1.0);
                        commands.spawn((
                            Sprite { image: assets.shot.clone(), ..default() },
                            Transform::from_xyz(pos.x, pos.y, 2.0)
                                .with_scale(Vec3::splat(SPRITE_SCALE)),
                            BoxCollider { size: Vec2::new(16.0, 16.0) },
                            Projectile {
                                vel_x: (dx / len) * BADDIE_SHOT_SPEED,
                                vel_y: (dy / len) * BADDIE_SHOT_SPEED,
                            },
                            LevelEntity,
                        ));
                        enemy.shoot_timer = fastrand_interval(CANNON_INTERVAL_MIN, CANNON_INTERVAL_MAX);
                    }
                }
            }

            EnemyKind::Cannon | EnemyKind::CannonBig | EnemyKind::SmallCannon => {
                if visible {
                    let imgs = match enemy.kind {
                        EnemyKind::CannonBig => &assets.cannon_big[..],
                        EnemyKind::SmallCannon => &assets.small_cannon[..],
                        _ => &assets.cannon[..],
                    };
                    let frame = (anim.frame / 12.0) as usize % 2;
                    sprite.image = imgs[frame].clone();
                    vel.x = 0.0;

                    let fully_out = pipe.map_or(true, |p| p.state == PipeDwellerState::Out);
                    if fully_out && enemy.shoot_timer <= 0.0 {
                        let dx = player_pos.x - pos.x;
                        let dir = if dx < 0.0 { -1.0 } else { 1.0 };
                        commands.spawn((
                            Sprite {
                                image: assets.cannon_bullet.clone(),
                                flip_x: dir < 0.0,
                                ..default()
                            },
                            Transform::from_xyz(pos.x, pos.y, 2.0)
                                .with_scale(Vec3::splat(SPRITE_SCALE)),
                            BoxCollider { size: Vec2::new(16.0, 12.0) },
                            Projectile {
                                vel_x: dir * SHOT_SPEED,
                                vel_y: 0.0,
                            },
                            LevelEntity,
                        ));
                        enemy.shoot_timer = fastrand_interval(CANNON_INTERVAL_MIN, CANNON_INTERVAL_MAX);
                    }
                }
            }

            EnemyKind::Boss => {
                if !enemy.dead {
                    let frame = (anim.frame / 8.0) as usize % 2;
                    let img = if enemy.hit_timer > 0.0 {
                        assets.bowser[(anim.frame as usize / 4) % 2 + 1].clone()
                    } else {
                        assets.bowser[frame].clone()
                    };
                    sprite.image = img;
                    sprite.flip_x = enemy.speed > 0.0;
                    vel.x = enemy.speed;
                } else {
                    enemy.die_timer -= dt;
                    vel.x = 0.0;
                }
            }
        }
    }
}

fn update_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut proj_q: Query<(Entity, &mut Transform, &Projectile), Without<MainCamera>>,
    camera_q: Query<&Transform, With<MainCamera>>,
) {
    let dt = time.delta_secs();
    let camera_x = camera_q.get_single().map(|t| t.translation.x).unwrap_or(320.0);

    for (entity, mut transform, proj) in proj_q.iter_mut() {
        transform.translation.x += proj.vel_x * dt;
        transform.translation.y += proj.vel_y * dt;

        if (transform.translation.x - camera_x).abs() > WINDOW_W * 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn stomp_detection(
    mut commands: Commands,
    mut player_q: Query<(&Transform, &BoxCollider, &mut Velocity, &mut Player)>,
    mut enemy_q: Query<
        (&Transform, &BoxCollider, &mut Enemy, &mut Sprite),
        Without<Player>,
    >,
    mut data: ResMut<GameData>,
    mut sound_events: EventWriter<SoundEvent>,
    assets: Res<GameAssets>,
) {
    let Ok((p_transform, p_coll, mut p_vel, _player)) = player_q.get_single_mut() else {
        return;
    };

    let p_pos = p_transform.translation.truncate();
    let p_half = p_coll.size * 0.5;
    let p_min = p_pos - p_half;
    let p_max = p_pos + p_half;

    for (e_transform, e_coll, mut enemy, mut sprite) in enemy_q.iter_mut() {
        if enemy.dead { continue; }
        if matches!(enemy.kind, EnemyKind::Cannon | EnemyKind::CannonBig | EnemyKind::SmallCannon) {
            continue;
        }

        let e_pos = e_transform.translation.truncate();
        let e_half = e_coll.size * 0.5;
        let e_min = e_pos - e_half;
        let e_max = e_pos + e_half;

        if p_min.x < e_max.x && p_max.x > e_min.x && p_min.y < e_max.y && p_max.y > e_min.y {
            let stomping = p_vel.y < 0.0 && p_pos.y - p_half.y < e_pos.y + e_half.y + 12.0;
            if stomping {
                data.score += if enemy.kind == EnemyKind::Boss {
                    SCORE_STOMP * 10
                } else {
                    SCORE_STOMP
                };
                p_vel.y = STOMP_BOUNCE;
                sound_events.send(SoundEvent::Stomp);

                if enemy.kind == EnemyKind::Boss {
                    enemy.hit_timer = 0.8;
                    enemy.hp -= 1;
                    if enemy.hp <= 0 {
                        enemy.dead = true;
                        enemy.die_timer = 3.5;
                    }
                } else {
                    commands.spawn((
                        Sprite { image: assets.exp[0].clone(), ..default() },
                        Transform::from_xyz(e_pos.x, e_pos.y, 5.0)
                            .with_scale(Vec3::splat(SPRITE_SCALE)),
                        EffectAnimation {
                            timer: 0.0,
                            frames: 3,
                            frame_duration: 0.08,
                            current_frame: 0,
                            sprites: vec![
                                assets.exp[0].clone(),
                                assets.exp[1].clone(),
                                assets.exp[2].clone(),
                            ],
                        },
                        LevelEntity,
                    ));
                    // Mark dead so enemy_player_collision skips it in the same
                    // frame; cleanup_dead_enemies (end of chain) despawns it.
                    enemy.dead = true;
                    enemy.die_timer = 0.0;
                    sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                }
            }
        }
    }
}

fn hazard_collision(
    hazard_q: Query<(&Transform, &BoxCollider), (With<Hazard>, Without<Player>)>,
    proj_q: Query<(Entity, &Transform, &BoxCollider), (With<Projectile>, Without<Player>)>,
    mut commands: Commands,
    mut player_q: Query<(&Transform, &BoxCollider, &mut Player)>,
    mut sound_events: EventWriter<SoundEvent>,
    mut data: ResMut<GameData>,
    mut next: ResMut<NextState<GameState>>,
    mut death_timer: ResMut<crate::states::DeathPauseTimer>,
) {
    let Ok((p_transform, p_coll, mut player)) = player_q.get_single_mut() else {
        return;
    };
    let p_pos = p_transform.translation.truncate();
    let p_min = p_pos - p_coll.size * 0.5;
    let p_max = p_pos + p_coll.size * 0.5;

    let mut take_hit = false;

    for (h_transform, h_coll) in hazard_q.iter() {
        let h_pos = h_transform.translation.truncate();
        let h_min = h_pos - h_coll.size * 0.5;
        let h_max = h_pos + h_coll.size * 0.5;
        if p_min.x < h_max.x && p_max.x > h_min.x && p_min.y < h_max.y && p_max.y > h_min.y {
            take_hit = true;
        }
    }

    for (entity, proj_t, proj_c) in proj_q.iter() {
        let p2_pos = proj_t.translation.truncate();
        let p2_min = p2_pos - proj_c.size * 0.5;
        let p2_max = p2_pos + proj_c.size * 0.5;
        if p_min.x < p2_max.x && p_max.x > p2_min.x && p_min.y < p2_max.y && p_max.y > p2_min.y {
            take_hit = true;
            commands.entity(entity).despawn();
        }
    }

    if take_hit && player.hit_timer <= 0.0 {
        player.hp -= 1;
        if player.hp <= 0 {
            sound_events.send(SoundEvent::Death);
            do_player_death(&mut data, &mut next, &mut death_timer);
        } else {
            player.hit_timer = 1.5; // longer invincibility when shrinking
            sound_events.send(SoundEvent::Stomp);
        }
    }
}

fn enemy_player_collision(
    enemy_q: Query<(&Transform, &BoxCollider, &Enemy), Without<Player>>,
    mut player_q: Query<(&Transform, &BoxCollider, &Velocity, &mut Player)>,
    mut data: ResMut<GameData>,
    mut next: ResMut<NextState<GameState>>,
    mut death_timer: ResMut<crate::states::DeathPauseTimer>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok((p_transform, p_coll, p_vel, mut player)) = player_q.get_single_mut() else {
        return;
    };
    if player.hit_timer > 0.0 { return; }

    let p_pos = p_transform.translation.truncate();
    let p_min = p_pos - p_coll.size * 0.5;
    let p_max = p_pos + p_coll.size * 0.5;

    for (e_transform, e_coll, enemy) in enemy_q.iter() {
        if enemy.dead { continue; }
        if matches!(enemy.kind, EnemyKind::Cannon | EnemyKind::CannonBig | EnemyKind::SmallCannon) {
            continue;
        }

        let e_pos = e_transform.translation.truncate();
        let e_min = e_pos - e_coll.size * 0.5;
        let e_max = e_pos + e_coll.size * 0.5;

        if p_min.x < e_max.x && p_max.x > e_min.x && p_min.y < e_max.y && p_max.y > e_min.y {
            let stomping = p_vel.y < 0.0
                && p_pos.y - p_coll.size.y * 0.5 < e_pos.y + e_coll.size.y * 0.5 + 12.0;
            if !stomping {
                player.hp -= 1;
                if player.hp <= 0 {
                    sound_events.send(SoundEvent::Death);
                    do_player_death(&mut data, &mut next, &mut death_timer);
                } else {
                    player.hit_timer = 1.5; // longer invincibility when shrinking
                    sound_events.send(SoundEvent::Stomp);
                }
            }
        }
    }
}

fn cleanup_dead_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Enemy)>,
) {
    for (entity, enemy) in query.iter() {
        if enemy.dead && enemy.die_timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn moving_platform_update(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut MovingPlatformV)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut vel, mut mplat) in query.iter_mut() {
        if transform.translation.y > mplat.origin_y + mplat.amplitude {
            mplat.speed = -mplat.speed.abs();
        }
        if transform.translation.y < mplat.origin_y - mplat.amplitude {
            mplat.speed = mplat.speed.abs();
        }
        vel.y = mplat.speed;
        transform.translation.y += vel.y * dt;
    }
}

fn platform_q_update(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sprite, &mut PlatformQ)>,
    assets: Res<GameAssets>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut sprite, mut pq) in query.iter_mut() {
        if pq.bump_timer > 0.0 {
            pq.bump_timer -= dt;
            if pq.bump_timer > 0.15 {
                transform.translation.y = pq.origin_y + 8.0 * SPRITE_SCALE;
            } else {
                transform.translation.y = pq.origin_y;
            }
        }
        let frame = if pq.used {
            2
        } else {
            (time.elapsed_secs() * (60.0 / 39.0)) as usize % 3
        };
        sprite.image = assets.platform_q[frame].clone();
    }
}

fn spring_animation(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut Spring)>,
    assets: Res<GameAssets>,
) {
    let dt = time.delta_secs();
    for (mut sprite, mut spring) in query.iter_mut() {
        spring.active_timer -= dt;
        sprite.image = if spring.active_timer > 0.0 {
            assets.spring[1].clone()
        } else {
            assets.spring[0].clone()
        };
    }
}

pub fn do_player_death(
    data: &mut ResMut<GameData>,
    next: &mut ResMut<NextState<GameState>>,
    death_timer: &mut ResMut<crate::states::DeathPauseTimer>,
) {
    data.lives -= 1;
    data.score = 0;
    death_timer.0 = 2.5;
    next.set(GameState::DeathPause);
}

fn pipe_dweller_update(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PipeDweller, &mut Visibility, &mut Enemy)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut pd, mut vis, mut enemy) in query.iter_mut() {
        let show_threshold = pd.emerged_y - 12.0;

        match pd.state {
            PipeDwellerState::Hidden => {
                transform.translation.y = pd.hidden_y;
                *vis = Visibility::Hidden;
                enemy.shoot_timer = enemy.shoot_timer.max(CANNON_INTERVAL_MIN);
                pd.timer -= dt;
                if pd.timer <= 0.0 {
                    pd.state = PipeDwellerState::Emerging;
                }
            }
            PipeDwellerState::Emerging => {
                transform.translation.y += PIPE_EMERGE_SPEED * dt;
                *vis = if transform.translation.y >= show_threshold {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if transform.translation.y >= pd.emerged_y {
                    transform.translation.y = pd.emerged_y;
                    pd.state = PipeDwellerState::Out;
                    pd.timer = PIPE_OUT_DURATION;
                }
            }
            PipeDwellerState::Out => {
                *vis = Visibility::Visible;
                pd.timer -= dt;
                if pd.timer <= 0.0 {
                    pd.state = PipeDwellerState::Retreating;
                }
            }
            PipeDwellerState::Retreating => {
                transform.translation.y -= PIPE_EMERGE_SPEED * dt;
                *vis = if transform.translation.y >= show_threshold {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if transform.translation.y <= pd.hidden_y {
                    transform.translation.y = pd.hidden_y;
                    pd.state = PipeDwellerState::Hidden;
                    pd.timer = PIPE_HIDDEN_DURATION;
                }
            }
        }
    }
}

fn fastrand_interval(min: f32, max: f32) -> f32 {
    (min + max) * 0.5
}
