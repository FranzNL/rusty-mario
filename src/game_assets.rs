use bevy::prelude::*;
use crate::states::GameState;

// Central resource holding all loaded asset handles
#[derive(Resource, Default)]
pub struct GameAssets {
    // Player
    pub mario: [Handle<Image>; 6],       // idle_r, walk1, walk2, walk3, walk4, jump
    pub mario_die: Handle<Image>,
    pub mario_life: Handle<Image>,

    // Enemies
    pub monster: [Handle<Image>; 3],     // walk1, walk2, dead_bounce/explode
    pub monster_red: [Handle<Image>; 3],
    pub slub: [Handle<Image>; 3],
    pub squidge: [Handle<Image>; 2],
    pub bowser: [Handle<Image>; 3],
    pub bowser_hit: Handle<Image>,
    pub cannon: [Handle<Image>; 2],
    pub cannon_big: [Handle<Image>; 2],
    pub small_cannon: [Handle<Image>; 2],

    // Projectiles / effects
    pub cannon_bullet: Handle<Image>,
    pub shot: Handle<Image>,
    pub exp: [Handle<Image>; 3],
    pub exp2: [Handle<Image>; 3],
    pub bowser_fireball: Handle<Image>,

    // Platforms & tiles
    pub platform_top: Handle<Image>,
    pub platform_middle: Handle<Image>,
    pub grass_top: Handle<Image>,
    pub grass_middle: Handle<Image>,
    pub brick1: Handle<Image>,
    pub brick2: Handle<Image>,
    pub gray1: Handle<Image>,
    pub gray2: Handle<Image>,
    pub platform_air: Handle<Image>,
    pub platform_q: [Handle<Image>; 3],
    pub platform_brick: Handle<Image>,
    pub pipe: Handle<Image>,
    pub pipe_big: Handle<Image>,
    pub moving_platform: Handle<Image>,
    pub moving_platform_long: Handle<Image>,
    pub bridge: Handle<Image>,
    pub spring: [Handle<Image>; 2],
    pub flagpole: Handle<Image>,

    // Decorative / world
    pub background1: Handle<Image>,
    pub background2: Handle<Image>,
    pub castle: Handle<Image>,
    pub castle_big: Handle<Image>,
    pub hill: Handle<Image>,
    pub bush: Handle<Image>,
    pub cloud: Handle<Image>,
    pub cloud2: Handle<Image>,
    pub fence: Handle<Image>,
    pub tree1: Handle<Image>,
    pub tree2: Handle<Image>,
    pub lava: Handle<Image>,
    pub wall: Handle<Image>,
    pub grass_texture: Handle<Image>,
    pub grass1: Handle<Image>,
    pub grass2: Handle<Image>,
    pub grass_sprite: Handle<Image>,
    pub chain: Handle<Image>,

    // Items
    pub coin: [Handle<Image>; 4],
    pub mushroom_green: Handle<Image>,
    pub flower: [Handle<Image>; 2],
    pub rose: [Handle<Image>; 2],

    // UI
    pub mario_icon: Handle<Image>,
    pub mario_life_icon: Handle<Image>,
    pub font: Handle<Font>,

    // Audio
    pub snd_jump: Handle<AudioSource>,
    pub snd_jump2: Handle<AudioSource>,
    pub snd_coin: Handle<AudioSource>,
    pub snd_stomp: Handle<AudioSource>,
    pub snd_death: Handle<AudioSource>,
    pub snd_1up: Handle<AudioSource>,
    pub snd_fireball: Handle<AudioSource>,
    pub mus_main: Handle<AudioSource>,
    pub mus_castle: Handle<AudioSource>,
    pub mus_goal: Handle<AudioSource>,
    pub mus_gameover: Handle<AudioSource>,
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(OnEnter(GameState::Loading), load_assets)
            .add_systems(Update, check_assets_ready.run_if(in_state(GameState::Loading)));
    }
}

fn load_assets(mut assets: ResMut<GameAssets>, server: Res<AssetServer>) {
    assets.mario = [
        server.load("sprites/mario1.png"),
        server.load("sprites/mario2.png"),
        server.load("sprites/mario3.png"),
        server.load("sprites/mario4.png"),
        server.load("sprites/mario1.png"), // walk cycle wraps
        server.load("sprites/mario5.png"), // jump
    ];
    assets.mario_die = server.load("sprites/mariodie.png");
    assets.mario_life = server.load("sprites/mario-life2.png");

    assets.monster = [
        server.load("sprites/monster1.png"),
        server.load("sprites/monster2.png"),
        server.load("sprites/monster3.png"),
    ];
    assets.monster_red = [
        server.load("sprites/monster-red1.png"),
        server.load("sprites/monster-red2.png"),
        server.load("sprites/monster-red3.png"),
    ];
    assets.slub = [
        server.load("sprites/slub1.png"),
        server.load("sprites/slub2.png"),
        server.load("sprites/slub3.png"),
    ];
    assets.squidge = [
        server.load("sprites/squidge1.png"),
        server.load("sprites/squidge2.png"),
    ];
    assets.bowser = [
        server.load("sprites/bowser1.png"),
        server.load("sprites/bowser2.png"),
        server.load("sprites/bowser3.png"),
    ];
    assets.bowser_hit = server.load("sprites/bowser3.png");
    assets.cannon = [
        server.load("sprites/cannon1.png"),
        server.load("sprites/cannon2.png"),
    ];
    assets.cannon_big = [
        server.load("sprites/cannonbig1.png"),
        server.load("sprites/cannonbig2.png"),
    ];
    assets.small_cannon = [
        server.load("sprites/smallcannon1.png"),
        server.load("sprites/smallcannon2.png"),
    ];
    assets.cannon_bullet = server.load("sprites/cannonbullet1.png");
    assets.shot = server.load("sprites/shot.png");
    assets.exp = [
        server.load("sprites/exp1.png"),
        server.load("sprites/exp2.png"),
        server.load("sprites/exp3.png"),
    ];
    assets.exp2 = [
        server.load("sprites/exp2-1.png"),
        server.load("sprites/exp2-2.png"),
        server.load("sprites/exp2-3.png"),
    ];
    assets.bowser_fireball = server.load("sprites/bowser-fireball1.png");

    assets.platform_top = server.load("sprites/platform-top.png");
    assets.platform_middle = server.load("sprites/platform-top.png");
    assets.grass_top = server.load("sprites/grass-1.png");
    assets.grass_middle = server.load("sprites/grass-middle.png");
    assets.brick1 = server.load("sprites/brick1.png");
    assets.brick2 = server.load("sprites/brick2.png");
    assets.gray1 = server.load("sprites/gray1.png");
    assets.gray2 = server.load("sprites/gray2.png");
    assets.platform_air = server.load("sprites/platform-air.png");
    assets.platform_q = [
        server.load("sprites/platform-q1.png"),
        server.load("sprites/platform-q2.png"),
        server.load("sprites/platform-q3.png"),
    ];
    assets.platform_brick = server.load("sprites/platform-brick.png");
    assets.pipe = server.load("sprites/pipe.png");
    assets.pipe_big = server.load("sprites/pipe-big.png");
    assets.moving_platform = server.load("sprites/moving-platform.png");
    assets.moving_platform_long = server.load("sprites/moving-platformlong.png");
    assets.bridge = server.load("sprites/bridge.png");
    assets.spring = [
        server.load("sprites/spring1.png"),
        server.load("sprites/spring2.png"),
    ];
    assets.flagpole = server.load("sprites/flagpole.png");

    assets.background1 = server.load("sprites/background-1.png");
    assets.background2 = server.load("sprites/background-2.png");
    assets.castle = server.load("sprites/castle.png");
    assets.castle_big = server.load("sprites/castle-big.png");
    assets.hill = server.load("sprites/hill.png");
    assets.bush = server.load("sprites/bush-1.png");
    assets.cloud = server.load("sprites/cloud.png");
    assets.cloud2 = server.load("sprites/dobbelclouds.png");
    assets.fence = server.load("sprites/fence.png");
    assets.tree1 = server.load("sprites/tree-1.png");
    assets.tree2 = server.load("sprites/tree-2.png");
    assets.lava = server.load("sprites/lava.png");
    assets.wall = server.load("sprites/wall-1.png");
    assets.grass_texture = server.load("sprites/grass-texture.png");
    assets.grass1 = server.load("sprites/grass-1.png");
    assets.grass2 = server.load("sprites/grass-2.png");
    assets.grass_sprite = server.load("sprites/grass-texturesprite.png");
    assets.chain = server.load("sprites/chain.png");

    assets.coin = [
        server.load("sprites/coin1.png"),
        server.load("sprites/coin2.png"),
        server.load("sprites/coin3.png"),
        server.load("sprites/coin4.png"),
    ];
    assets.mushroom_green = server.load("sprites/mushroom-green.png");
    assets.flower = [
        server.load("sprites/flower1.png"),
        server.load("sprites/flower2.png"),
    ];
    assets.rose = [
        server.load("sprites/rose1.png"),
        server.load("sprites/rose2.png"),
    ];

    assets.mario_icon = server.load("sprites/mario5.png");
    assets.mario_life_icon = server.load("sprites/mario-life2.png");
    assets.font = server.load("fonts/font.ttf");

    assets.snd_jump = server.load("audio/jump.ogg");
    assets.snd_jump2 = server.load("audio/jump2.ogg");
    assets.snd_coin = server.load("audio/coin.ogg");
    assets.snd_stomp = server.load("audio/stomp.ogg");
    assets.snd_death = server.load("audio/death.ogg");
    assets.snd_1up = server.load("audio/1up.ogg");
    assets.snd_fireball = server.load("audio/fireball.ogg");
    assets.mus_main = server.load("audio/maintheme.ogg");
    assets.mus_castle = server.load("audio/castle.ogg");
    assets.mus_goal = server.load("audio/goal.ogg");
    assets.mus_gameover = server.load("audio/gameover.ogg");
}

fn check_assets_ready(
    mut next: ResMut<NextState<GameState>>,
    assets: Res<GameAssets>,
    server: Res<AssetServer>,
) {
    use bevy::asset::LoadState;
    let key_images: &[&Handle<Image>] = &[
        &assets.mario[0],
        &assets.platform_top,
        &assets.background2,
    ];
    let key_audio: &[&Handle<AudioSource>] = &[
        &assets.mus_main,
        &assets.snd_jump,
    ];
    let images_ready = key_images.iter().all(|h| {
        matches!(server.get_load_state(h.id()), Some(LoadState::Loaded))
    });
    let audio_ready = key_audio.iter().all(|h| {
        matches!(server.get_load_state(h.id()), Some(LoadState::Loaded))
    });
    if images_ready && audio_ready {
        next.set(GameState::MainMenu);
    }
}
