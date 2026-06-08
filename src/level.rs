use bevy::prelude::*;
use image::GenericImageView;
use crate::components::*;
use crate::constants::*;
use crate::game_assets::GameAssets;
use crate::states::GameState;

// Level PNG bytes embedded at compile time for WASM compatibility
const LEVEL1: &[u8] = include_bytes!("../assets/levels/lvl1.png");
const LEVEL2: &[u8] = include_bytes!("../assets/levels/lvl2.png");
const LEVEL3: &[u8] = include_bytes!("../assets/levels/lvl3.png");
const LEVEL4: &[u8] = include_bytes!("../assets/levels/lvl4.png");

#[derive(Resource)]
pub struct LevelMeta {
    pub width_px: f32,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelMeta { width_px: 6048.0 })
            .add_systems(OnEnter(GameState::Playing), spawn_level)
            .add_systems(
                OnExit(GameState::Playing),
                (apply_deferred, cleanup_level_entities).chain(),
            );
    }
}

fn cleanup_level_entities(mut commands: Commands, q: Query<Entity, With<LevelEntity>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

fn level_bytes(lvl: u32) -> &'static [u8] {
    match lvl {
        1 => LEVEL1,
        2 => LEVEL2,
        3 => LEVEL3,
        _ => LEVEL4,
    }
}

fn spawn_level(
    mut commands: Commands,
    assets: Res<GameAssets>,
    data: Res<crate::components::GameData>,
    mut meta: ResMut<LevelMeta>,
) {
    let bytes = level_bytes(data.level);
    let img = image::load_from_memory(bytes).expect("level png invalid");
    let w = img.width();
    let h = img.height();
    meta.width_px = w as f32 * TILE;

    // Spawn scrolling background (3 tiles wide for parallax wrap)
    let bg_handle = if data.level >= 5 {
        assets.background1.clone()
    } else {
        assets.background2.clone()
    };
    for i in -1..=2i32 {
        commands.spawn((
            Sprite { image: bg_handle.clone(), ..default() },
            Transform::from_xyz(
                i as f32 * WINDOW_W + WINDOW_W * 0.5,
                CAMERA_Y,
                -1.0,
            ).with_scale(Vec3::new(1.0, 1.0, 1.0)), // backgrounds are 640×480 already
            Background,
            LevelEntity,
        ));
    }

    for row in 0..h {
        for col in 0..w {
            let rgba = img.get_pixel(col, row);
            let r = rgba[0];
            let g = rgba[1];
            let b = rgba[2];

            // World position for tile centers (Y-flipped from PNG coords)
            let wx = col as f32 * TILE + TILE * 0.5;
            let wy = -(row as f32 * TILE + TILE * 0.5);

            match (r, g, b) {
                // ── Solid platform tiles ──────────────────────────────
                (0, 0, 0) => {
                    let above = row > 0 && pixel_rgb(&img, col, row - 1) == [0, 0, 0];
                    let img_h = if above { assets.platform_middle.clone() } else { assets.platform_top.clone() };
                    spawn_solid_tile(&mut commands, &assets, img_h, wx, wy, TILE, TILE);
                }
                (0, 19, 127) => {
                    let above = row > 0 && pixel_rgb(&img, col, row - 1) == [0, 19, 127];
                    let img_h = if above { assets.grass_middle.clone() } else { assets.grass_top.clone() };
                    spawn_solid_tile(&mut commands, &assets, img_h, wx, wy, TILE, TILE);
                }
                (109, 127, 63) => {
                    let above = row > 0 && pixel_rgb(&img, col, row - 1) == [109, 127, 63];
                    let img_h = if above { assets.brick2.clone() } else { assets.brick1.clone() };
                    spawn_solid_tile(&mut commands, &assets, img_h, wx, wy, TILE, TILE);
                }
                (48, 48, 48) => {
                    let above = row > 0 && pixel_rgb(&img, col, row - 1) == [48, 48, 48];
                    let img_h = if above { assets.gray2.clone() } else { assets.gray1.clone() };
                    spawn_solid_tile(&mut commands, &assets, img_h, wx, wy, TILE, TILE);
                }
                (255, 200, 0) => {
                    spawn_solid_tile(&mut commands, &assets, assets.platform_air.clone(), wx, wy, TILE, TILE);
                }
                (0, 74, 127) => {
                    spawn_solid_tile(&mut commands, &assets, assets.platform_brick.clone(), wx, wy, TILE, TILE);
                }
                (0, 127, 127) => {
                    spawn_solid_tile(&mut commands, &assets, assets.grass_texture.clone(), wx, wy, TILE, TILE);
                }
                (255, 0, 110) => {
                    spawn_solid_tile(&mut commands, &assets, assets.grass1.clone(), wx, wy, TILE, TILE);
                }
                (165, 255, 127) => {
                    spawn_solid_tile(&mut commands, &assets, assets.grass2.clone(), wx, wy, TILE, TILE);
                }
                (127, 89, 63) => {
                    spawn_solid_tile(&mut commands, &assets, assets.bridge.clone(), wx, wy, TILE, TILE);
                }

                // PlatformQ (question block)
                (127, 51, 0) => {
                    commands.spawn((
                        Sprite { image: assets.platform_q[0].clone(), ..default() },
                        Transform::from_xyz(wx, wy, 1.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(TILE, TILE) },
                        Solid,
                        PlatformQ { used: false, bump_timer: 0.0, origin_y: wy },
                        LevelEntity,
                    ));
                }

                // ── Pipes ────────────────────────────────────────────
                // pipe.png: 32×48 px → rendered 64×96 at SPRITE_SCALE=2.
                // Level pixel marks the bottom tile of the pipe (just above ground).
                // Center x = left-column + 1 tile (pipe is 2 tiles wide).
                // Center y: bottom at ground top → center = -(row*TILE) + TILE*0.5
                (91, 127, 0) => {
                    let px = col as f32 * TILE + TILE;
                    let py = -(row as f32 * TILE) + TILE * 0.5;
                    commands.spawn((
                        Sprite { image: assets.pipe.clone(), ..default() },
                        Transform::from_xyz(px, py, 1.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(TILE * 2.0, TILE * 3.0) },
                        Solid,
                        LevelEntity,
                    ));
                }
                // pipe-big.png: 32×64 px → rendered 64×128 at SPRITE_SCALE=2.
                // Center y: bottom at ground top → center = -(row*TILE) + TILE
                (178, 0, 255) => {
                    let px = col as f32 * TILE + TILE;
                    let py = -(row as f32 * TILE) + TILE;
                    commands.spawn((
                        Sprite { image: assets.pipe_big.clone(), ..default() },
                        Transform::from_xyz(px, py, 1.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(TILE * 2.0, TILE * 4.0) },
                        Solid,
                        LevelEntity,
                    ));
                }

                // ── Moving platforms ──────────────────────────────────
                (255, 0, 0) => {
                    commands.spawn((
                        Sprite { image: assets.moving_platform.clone(), ..default() },
                        Transform::from_xyz(wx, wy, 1.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(TILE * 2.0, TILE * 0.5) },
                        Solid,
                        Velocity { x: 0.0, y: 60.0 },
                        MovingPlatformV { origin_y: wy, speed: 60.0, amplitude: 64.0 },
                        LevelEntity,
                    ));
                }
                (82, 127, 63) => {
                    commands.spawn((
                        Sprite { image: assets.moving_platform_long.clone(), ..default() },
                        Transform::from_xyz(wx, wy, 1.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(TILE * 4.0, TILE * 0.5) },
                        Solid,
                        Velocity { x: 0.0, y: 60.0 },
                        MovingPlatformV { origin_y: wy, speed: 60.0, amplitude: 64.0 },
                        LevelEntity,
                    ));
                }

                // ── Springs ───────────────────────────────────────────
                (0, 200, 0) => {
                    commands.spawn((
                        Sprite { image: assets.spring[0].clone(), ..default() },
                        Transform::from_xyz(wx, wy, 1.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(TILE, TILE * 0.75) },
                        Solid,
                        Spring { active_timer: 0.0 },
                        LevelEntity,
                    ));
                }

                // ── Items & pickups ───────────────────────────────────
                (255, 255, 0) => {
                    // Skip coins sitting directly above a Q-block — bumping the box gives the coin
                    let above_q = row + 1 < h && pixel_rgb(&img, col, row + 1) == [127, 51, 0];
                    if !above_q {
                        commands.spawn((
                            Sprite { image: assets.coin[0].clone(), ..default() },
                            Transform::from_xyz(wx + 4.0, wy, 2.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                            BoxCollider { size: Vec2::new(16.0, 28.0) },
                            Coin { frame: 0.0 },
                            LevelEntity,
                        ));
                    }
                }
                (0, 127, 70) => {
                    commands.spawn((
                        Sprite { image: assets.mushroom_green.clone(), ..default() },
                        Transform::from_xyz(wx, wy, 2.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(28.0, 28.0) },
                        MushroomGreen,
                        LevelEntity,
                    ));
                }

                // Level exit (flagpole). Sprite is 24×168 px; at SPRITE_SCALE=2 world height=336.
                // Ground surface sits at y = -(13 * TILE) = -416; center = ground + half_world_h.
                (0, 255, 0) | (128, 128, 128) => {
                    let ground_y = -(13.0 * TILE);
                    let py = ground_y + 168.0; // 168 = 336/2 (world half-height)
                    commands.spawn((
                        Sprite { image: assets.flagpole.clone(), ..default() },
                        Transform::from_xyz(wx, py, 2.0)
                            .with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(TILE, TILE * 4.0) },
                        LevelExit,
                        LevelEntity,
                    ));
                }

                // ── Enemies ───────────────────────────────────────────
                (0, 0, 255) => spawn_baddie(&mut commands, &assets, wx + 2.0, wy + 4.0, EnemyKind::Monster),
                (0, 255, 255) => spawn_baddie(&mut commands, &assets, wx + 1.0, wy + 2.0, EnemyKind::Slub),
                (255, 0, 255) => spawn_baddie(&mut commands, &assets, wx, wy, EnemyKind::Squidge),
                (255, 106, 0) => spawn_baddie(&mut commands, &assets, wx + 2.0, wy + 4.0, EnemyKind::MonsterRed),

                (76, 255, 0) => spawn_cannon(&mut commands, &assets, wx, -(row as f32 * 29.3 + 4.0), EnemyKind::Cannon),
                (63, 73, 127) => spawn_cannon(&mut commands, &assets, wx, -(row as f32 * 24.5 + 4.0), EnemyKind::CannonBig),
                (255, 127, 182) => spawn_cannon(&mut commands, &assets, wx, wy + 2.0, EnemyKind::SmallCannon),

                (200, 0, 0) => {
                    let py = -(row as f32 * TILE) - 31.0;
                    commands.spawn((
                        Sprite { image: assets.bowser[0].clone(), ..default() },
                        Transform::from_xyz(wx, py, 2.0).with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(60.0, 60.0) },
                        Velocity { x: -BOSS_SPEED, y: 0.0 },
                        Grounded::default(),
                        AffectedByGravity,
                        Enemy::new(EnemyKind::Boss, -BOSS_SPEED),
                        AnimationState::default(),
                        LevelEntity,
                    ));
                }

                // ── Hazards ───────────────────────────────────────────
                (127, 0, 110) => {
                    let plant_h = 48.0f32;
                    let emerged_y = wy + TILE;
                    let hidden_y = emerged_y - plant_h;
                    commands.spawn((
                        Sprite {
                            image: assets.flower[0].clone(),
                            color: Color::srgba(1., 1., 1., 0.),
                            ..default()
                        },
                        Transform::from_xyz(col as f32 * 32.5 + 16.0, hidden_y, 0.5)
                            .with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(28.0, plant_h) },
                        Hazard,
                        FlowerPlant,
                        PipeDweller {
                            emerged_y,
                            hidden_y,
                            state: PipeDwellerState::Hidden,
                            timer: fastrand_range(PIPE_HIDDEN_DURATION, PIPE_HIDDEN_DURATION + 2.0),
                        },
                        LevelEntity,
                    ));
                }
                (72, 0, 255) => {
                    let plant_h = 48.0f32;
                    let emerged_y = wy + TILE;
                    let hidden_y = emerged_y - plant_h;
                    commands.spawn((
                        Sprite {
                            image: assets.rose[0].clone(),
                            color: Color::srgba(1., 1., 1., 0.),
                            ..default()
                        },
                        Transform::from_xyz(col as f32 * 32.23 + 16.0, hidden_y, 0.5)
                            .with_scale(Vec3::splat(SPRITE_SCALE)),
                        BoxCollider { size: Vec2::new(28.0, plant_h) },
                        Hazard,
                        RosePlant,
                        PipeDweller {
                            emerged_y,
                            hidden_y,
                            state: PipeDwellerState::Hidden,
                            timer: fastrand_range(PIPE_HIDDEN_DURATION, PIPE_HIDDEN_DURATION + 2.0),
                        },
                        LevelEntity,
                    ));
                }

                // ── Decorative (no physics) ───────────────────────────
                (87, 0, 127) => decor(&mut commands, assets.cloud.clone(), wx, wy, 1.0),
                (127, 0, 55) => decor(&mut commands, assets.bush.clone(), wx, wy, 1.0),
                // Sprites are at SPRITE_SCALE=2; center y = ground_surface + pixel_height.
                // Ground surface = -(13 * TILE). pixel_h values: castle=80, castle_big=176, hill=35.
                (80, 63, 127) => decor(&mut commands, assets.castle.clone(), wx, -(13.0 * TILE) + 80.0, 0.5),
                (214, 127, 255) => decor(&mut commands, assets.castle_big.clone(), wx, -(13.0 * TILE) + 176.0, 0.5),
                (255, 233, 127) => decor(&mut commands, assets.hill.clone(), wx, -(13.0 * TILE) + 35.0, 0.5),
                (64, 64, 64) => decor(&mut commands, assets.fence.clone(), wx, wy, 1.5),
                // pixel_h: tree1=46, tree2=30
                (182, 255, 0) => decor(&mut commands, assets.tree1.clone(), wx, -(13.0 * TILE) + 46.0, 0.5),
                (38, 127, 0) => decor(&mut commands, assets.tree2.clone(), wx, -(13.0 * TILE) + 30.0, 0.5),
                (255, 0, 220) => decor(&mut commands, assets.cloud2.clone(), wx, wy, 0.5),
                (255, 127, 127) => decor(&mut commands, assets.grass_sprite.clone(), wx, wy, 1.5),
                (127, 255, 197) => decor(&mut commands, assets.wall.clone(), wx, -(row as f32 * 19.0), 0.5),
                (234, 106, 68) => decor(&mut commands, assets.lava.clone(), wx, wy, 1.5),
                (127, 116, 63) => decor(&mut commands, assets.chain.clone(), wx, wy, 1.5),

                _ => {}
            }
        }
    }
}

fn spawn_solid_tile(
    commands: &mut Commands,
    _assets: &GameAssets,
    image: Handle<Image>,
    x: f32,
    y: f32,
    cw: f32,
    ch: f32,
) {
    commands.spawn((
        Sprite { image, ..default() },
        Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(SPRITE_SCALE)),
        BoxCollider { size: Vec2::new(cw, ch) },
        Solid,
        LevelEntity,
    ));
}

fn spawn_baddie(
    commands: &mut Commands,
    assets: &GameAssets,
    x: f32,
    y: f32,
    kind: EnemyKind,
) {
    let image = match kind {
        EnemyKind::Slub => assets.slub[0].clone(),
        EnemyKind::Squidge => assets.squidge[0].clone(),
        EnemyKind::MonsterRed => assets.monster_red[0].clone(),
        _ => assets.monster[0].clone(),
    };
    let speed = if kind == EnemyKind::Slub {
        ENEMY_SPEED * 2.0
    } else {
        ENEMY_SPEED
    };
    commands.spawn((
        Sprite { image, ..default() },
        Transform::from_xyz(x, y, 2.0).with_scale(Vec3::splat(SPRITE_SCALE)),
        BoxCollider { size: Vec2::new(28.0, 28.0) },
        Velocity { x: speed, y: 0.0 },
        Grounded::default(),
        AffectedByGravity,
        Enemy::new(kind, speed),
        AnimationState::default(),
        LevelEntity,
    ));
}

fn spawn_cannon(
    commands: &mut Commands,
    assets: &GameAssets,
    x: f32,
    y: f32,
    kind: EnemyKind,
) {
    let image = match kind {
        EnemyKind::CannonBig => assets.cannon_big[0].clone(),
        EnemyKind::SmallCannon => assets.small_cannon[0].clone(),
        _ => assets.cannon[0].clone(),
    };
    let size = match kind {
        EnemyKind::CannonBig => Vec2::new(48.0, 56.0),
        _ => Vec2::new(28.0, 42.0),
    };
    let mut enemy = Enemy::new(kind, 0.0);
    enemy.shoot_timer = fastrand_range(CANNON_INTERVAL_MIN, CANNON_INTERVAL_MAX);

    // Start hidden inside the pipe; emerged_y is the fully-out position
    let hidden_y = y - size.y;
    commands.spawn((
        Sprite { image, ..default() },
        Transform::from_xyz(x, hidden_y, 2.0).with_scale(Vec3::splat(SPRITE_SCALE)),
        BoxCollider { size },
        Solid,
        Visibility::Hidden,
        Enemy { kind, ..enemy },
        AnimationState::default(),
        PipeDweller {
            emerged_y: y,
            hidden_y,
            state: PipeDwellerState::Hidden,
            timer: fastrand_range(PIPE_HIDDEN_DURATION, PIPE_HIDDEN_DURATION + 1.5),
        },
        LevelEntity,
    ));
}

fn decor(commands: &mut Commands, image: Handle<Image>, x: f32, y: f32, z: f32) {
    commands.spawn((
        Sprite { image, ..default() },
        Transform::from_xyz(x, y, z).with_scale(Vec3::splat(SPRITE_SCALE)),
        LevelEntity,
    ));
}

fn fastrand_range(min: f32, max: f32) -> f32 {
    min + (max - min) * 0.5
}

fn pixel_rgb(img: &image::DynamicImage, col: u32, row: u32) -> [u8; 3] {
    let p = img.get_pixel(col, row);
    [p[0], p[1], p[2]]
}
