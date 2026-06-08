use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::game_assets::GameAssets;
use crate::physics::PhysicsSet;
use crate::states::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    player_input,
                    player_animation,
                    player_off_screen,
                )
                    .chain()
                    .after(PhysicsSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_player);
    }
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Sprite {
            image: assets.mario[0].clone(),
            ..default()
        },
        Transform::from_xyz(PLAYER_START_X, PLAYER_START_Y, 3.0)
            .with_scale(Vec3::splat(SPRITE_SCALE)),
        BoxCollider {
            size: Vec2::new(PLAYER_W, PLAYER_H),
        },
        Velocity::default(),
        Grounded::default(),
        AffectedByGravity,
        Player::default(),
        AnimationState::default(),
        LevelEntity,
    ));
}

fn despawn_player(mut commands: Commands, q: Query<Entity, With<Player>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

pub fn player_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    vi: Res<crate::components::VirtualInput>,
    mut query: Query<(&mut Velocity, &mut Grounded, &mut Player, &Transform)>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok((mut vel, mut grounded, mut player, _transform)) = query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    player.hit_timer -= dt;
    player.still_timer -= dt;

    let jump_pressed = keys.just_pressed(KeyCode::KeyZ) || vi.jump_just;
    let jump_held = keys.pressed(KeyCode::KeyZ) || vi.jump;
    player.holding_jump = jump_held;

    // Buffer a jump press for up to 100ms so timing windows feel responsive
    if jump_pressed {
        player.jump_buffer = 0.1;
    } else {
        player.jump_buffer = (player.jump_buffer - dt).max(0.0);
    }

    // Horizontal movement
    let mut dx = 0.0f32;
    if player.still_timer <= 0.0 {
        if keys.pressed(KeyCode::ArrowLeft) || vi.left {
            dx = -1.0;
            player.facing = -1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) || vi.right {
            dx = 1.0;
            player.facing = 1.0;
        }
    }
    vel.x = dx * H_SPEED;

    // Land detection (runs against fresh grounded state from this frame's physics)
    if grounded.0 {
        player.jumping = false;
        player.springing = false;
    }

    // Jump: fire if grounded and a recent press is buffered
    if player.jump_buffer > 0.0 && grounded.0 && player.still_timer <= 0.0 {
        vel.y = JUMP_VY;
        player.jumping = true;
        player.springing = false;
        player.jump_buffer = 0.0;
        grounded.0 = false;
        sound_events.send(SoundEvent::Jump);
    }

    // Track airborne state for animation
    if vel.y < -GRAVITY_RELEASE / 60.0 {
        player.jumping = true;
    }

    // Hit timer: blink and brief invincibility
    // (handled in animation below)
}

pub fn player_animation(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut AnimationState, &Velocity, &Player, &mut Transform, &mut BoxCollider)>,
    assets: Res<GameAssets>,
) {
    let Ok((mut sprite, mut anim, vel, player, mut transform, mut collider)) = query.get_single_mut() else {
        return;
    };

    anim.frame += time.delta_secs() * 60.0;

    let big = player.hp >= 2;

    // Scale and collider track power state
    transform.scale = if big { Vec3::splat(3.0) } else { Vec3::splat(SPRITE_SCALE) };
    collider.size = if big {
        Vec2::new(PLAYER_W, PLAYER_H * 1.5)
    } else {
        Vec2::new(PLAYER_W, PLAYER_H)
    };

    let moving = vel.x.abs() > 1.0;
    let flipped = player.facing < 0.0;

    let image = if player.jumping {
        assets.mario[5].clone()
    } else if moving {
        let walk_frame = (anim.frame / 6.0) as usize % 5;
        assets.mario[walk_frame].clone()
    } else {
        assets.mario[0].clone()
    };

    // Blink when hit
    if player.hit_timer > 0.0 && (anim.frame as i32 % 4 < 2) {
        sprite.image = assets.mario[2].clone();
        sprite.color = Color::WHITE;
    } else {
        sprite.image = image;
        sprite.color = if big {
            Color::srgb(1.0, 0.85, 0.3) // golden tint when powered up
        } else {
            Color::WHITE
        };
    }

    sprite.flip_x = flipped;
}

fn player_off_screen(
    query: Query<&Transform, With<Player>>,
    mut data: ResMut<GameData>,
    mut next: ResMut<NextState<GameState>>,
    mut death_timer: ResMut<crate::states::DeathPauseTimer>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok(transform) = query.get_single() else { return; };
    if transform.translation.y < -(LEVEL_HEIGHT_PX + 100.0) {
        sound_events.send(SoundEvent::Death);
        crate::enemies::do_player_death(&mut data, &mut next, &mut death_timer);
    }
}
