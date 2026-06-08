use bevy::prelude::*;
use crate::components::VirtualInput;
use crate::game_assets::GameAssets;
use crate::states::GameState;
use crate::player::player_input;

pub struct TouchControlsPlugin;

impl Plugin for TouchControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VirtualInput>()
            .add_systems(OnEnter(GameState::MainMenu), setup_menu_button)
            .add_systems(OnExit(GameState::MainMenu), cleanup_touch_buttons)
            .add_systems(OnEnter(GameState::Playing), setup_touch_buttons)
            .add_systems(OnExit(GameState::Playing), cleanup_touch_buttons)
            .add_systems(
                Update,
                (update_virtual_input.before(player_input))
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)] struct TouchBtnLeft;
#[derive(Component)] struct TouchBtnRight;
#[derive(Component)] struct TouchBtnJump;
#[derive(Component)] struct TouchControl;

const BTN_SIZE: f32 = 55.0;
const BTN_GAP: f32 = 10.0;
const BTN_MARGIN: f32 = 6.0;
const BTN_COLOR: Color = Color::srgba(0.05, 0.05, 0.05, 0.55);
const BTN_PRESSED: Color = Color::srgba(0.9, 0.9, 0.9, 0.55);

fn spawn_btn(
    parent: &mut ChildBuilder,
    label: &str,
    font: Handle<Font>,
    left: Option<f32>,
    right: Option<f32>,
    extra: impl Bundle,
) {
    let mut node = Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(BTN_MARGIN),
        width: Val::Px(BTN_SIZE),
        height: Val::Px(BTN_SIZE),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    if let Some(l) = left { node.left = Val::Px(l); }
    if let Some(r) = right { node.right = Val::Px(r); }

    parent.spawn((
        node,
        Button,
        BackgroundColor(BTN_COLOR),
        BorderRadius::all(Val::Px(8.0)),
        extra,
    )).with_children(|p| {
        p.spawn((
            Text::new(label),
            TextFont { font, font_size: 22.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn setup_touch_buttons(mut commands: Commands, assets: Res<GameAssets>) {
    let font = assets.font.clone();
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        TouchControl,
        PickingBehavior { should_block_lower: false, is_hoverable: false },
    )).with_children(|parent| {
        spawn_btn(parent, "<", font.clone(), Some(BTN_MARGIN), None, TouchBtnLeft);
        spawn_btn(parent, ">", font.clone(), Some(BTN_MARGIN + BTN_SIZE + BTN_GAP), None, TouchBtnRight);
        spawn_btn(parent, "Z", font.clone(), None, Some(BTN_MARGIN), TouchBtnJump);
    });
}

fn cleanup_touch_buttons(mut commands: Commands, q: Query<Entity, With<TouchControl>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn setup_menu_button(mut commands: Commands, assets: Res<GameAssets>) {
    let font = assets.font.clone();
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        TouchControl,
        PickingBehavior { should_block_lower: false, is_hoverable: false },
    )).with_children(|parent| {
        spawn_btn(parent, "Z", font, None, Some(BTN_MARGIN), TouchBtnJump);
    });
}

type TouchBtnFilter = Or<(With<TouchBtnLeft>, With<TouchBtnRight>, With<TouchBtnJump>)>;

fn update_virtual_input(
    mut vi: ResMut<VirtualInput>,
    mut prev_jump: Local<bool>,
    mut buttons: Query<(
        &Interaction,
        &mut BackgroundColor,
        Option<&TouchBtnLeft>,
        Option<&TouchBtnRight>,
        Option<&TouchBtnJump>,
    ), TouchBtnFilter>,
) {
    vi.left = false;
    vi.right = false;
    vi.jump = false;

    for (interaction, mut bg, left, right, jump) in &mut buttons {
        let pressed = *interaction == Interaction::Pressed;
        bg.0 = if pressed { BTN_PRESSED } else { BTN_COLOR };
        if left.is_some() { vi.left = pressed; }
        if right.is_some() { vi.right = pressed; }
        if jump.is_some() { vi.jump = pressed; }
    }

    vi.jump_just = vi.jump && !*prev_jump;
    *prev_jump = vi.jump;
}
