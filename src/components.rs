use bevy::prelude::*;

// ── Movement & Physics ──────────────────────────────────────────────
#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct BoxCollider {
    pub size: Vec2,
}

#[derive(Component, Default)]
pub struct Grounded(pub bool);

#[derive(Component)]
pub struct AffectedByGravity;

// ── Entity Tags ─────────────────────────────────────────────────────
#[derive(Component)]
pub struct Player {
    pub facing: f32, // 1.0 = right, -1.0 = left
    pub jumping: bool,
    pub hp: i32,
    pub hit_timer: f32,
    pub still_timer: f32,
    pub holding_jump: bool,
    pub springing: bool,
    pub jump_buffer: f32, // seconds remaining to fire a buffered jump
}

impl Default for Player {
    fn default() -> Self {
        Self {
            facing: 1.0,
            jumping: false,
            hp: 1,
            hit_timer: 0.0,
            still_timer: 0.0,
            holding_jump: false,
            springing: false,
            jump_buffer: 0.0,
        }
    }
}

#[derive(Component)]
pub struct AnimationState {
    pub frame: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self { frame: 0.0 }
    }
}

// ── Platforms & Solids ──────────────────────────────────────────────
#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct MovingPlatformV {
    pub origin_y: f32,
    pub speed: f32,
    pub amplitude: f32, // ±pixels
}

#[derive(Component)]
pub struct PlatformQ {
    pub used: bool,
    pub bump_timer: f32,
    pub origin_y: f32,
}

// ── Enemies ─────────────────────────────────────────────────────────
#[derive(Component, Clone, Copy, PartialEq)]
pub enum EnemyKind {
    Monster,
    Slub,
    Squidge,
    MonsterRed,
    Cannon,
    CannonBig,
    SmallCannon,
    Boss,
}

#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub speed: f32, // signed: positive = right
    pub hp: i32,
    pub hit_timer: f32,
    pub dead: bool,
    pub die_timer: f32,
    pub shoot_timer: f32, // cooldown for cannons/squidge
}

impl Enemy {
    pub fn new(kind: EnemyKind, speed: f32) -> Self {
        let hp = if kind == EnemyKind::Boss { 5 } else { 1 };
        Self {
            kind,
            speed,
            hp,
            hit_timer: 0.0,
            dead: false,
            die_timer: 0.0,
            shoot_timer: 0.0,
        }
    }
}

// ── Projectiles ─────────────────────────────────────────────────────
#[derive(Component)]
pub struct Projectile {
    pub vel_x: f32,
    pub vel_y: f32,
}

// ── Pipe dwelling ────────────────────────────────────────────────────
#[derive(PartialEq, Clone, Copy)]
pub enum PipeDwellerState {
    Hidden,
    Emerging,
    Out,
    Retreating,
}

#[derive(Component)]
pub struct PipeDweller {
    pub emerged_y: f32,
    pub hidden_y: f32,
    pub state: PipeDwellerState,
    pub timer: f32,
}

// ── Items & Interactive ─────────────────────────────────────────────
#[derive(Component)]
pub struct Coin {
    pub frame: f32,
}

#[derive(Component)]
pub struct Spring {
    pub active_timer: f32,
}

#[derive(Component)]
pub struct LevelExit; // bomb/flag — touching advances level

#[derive(Component)]
pub struct MushroomGreen;

#[derive(Component)]
pub struct Hazard; // flowers, roses, lava — touching damages player

#[derive(Component)]
pub struct RosePlant;

#[derive(Component)]
pub struct FlowerPlant;

// ── Effects ────────────────────────────────────────────────────────
#[derive(Component)]
pub struct DeathMario {
    pub vel_y: f32,
}

#[derive(Component)]
pub struct EffectAnimation {
    pub timer: f32,
    pub frames: usize,
    pub frame_duration: f32,
    pub current_frame: usize,
    pub sprites: Vec<Handle<Image>>,
}

// ── Background / Decorative ─────────────────────────────────────────
#[derive(Component)]
pub struct Background; // parallax background sprite

// ── Marker for cleanup ─────────────────────────────────────────────
#[derive(Component)]
pub struct LevelEntity; // all level-specific entities get this for cleanup

#[derive(Component)]
pub struct UiEntity;

// ── Camera marker ───────────────────────────────────────────────────
#[derive(Component)]
pub struct MainCamera;

// ── Block bump event ────────────────────────────────────────────────
#[derive(Event)]
pub struct BumpEvent(pub Entity);

#[derive(Component)]
pub struct PopCoin {
    pub vel_y: f32,
    pub timer: f32,
    pub frame: f32,
}

// ── Virtual (touch/mouse) input ─────────────────────────────────────
#[derive(Resource, Default)]
pub struct VirtualInput {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub jump_just: bool,
}

// ── Sound Events ────────────────────────────────────────────────────
#[derive(Event, Clone, Copy, PartialEq)]
pub enum SoundEvent {
    Jump,
    Coin,
    Stomp,
    Death,
    Up,
}

// ── Game state data ─────────────────────────────────────────────────
#[derive(Resource, Default)]
pub struct GameData {
    pub score: u32,
    pub highscore: u32,
    pub lives: i32,
    pub level: u32,
    pub time: f32,
}

// ── UI references ───────────────────────────────────────────────────
#[derive(Resource, Default)]
pub struct UiHandles {
    pub score_text: Option<Entity>,
    pub lives_text: Option<Entity>,
    pub world_text: Option<Entity>,
    pub time_text: Option<Entity>,
    pub mario_icon: Option<Entity>,
}
