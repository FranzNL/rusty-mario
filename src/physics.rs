use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::states::GameState;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicsSet;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BumpEvent>()
            .configure_sets(Update, PhysicsSet.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                (apply_gravity, move_entities)
                    .chain()
                    .in_set(PhysicsSet),
            );
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Grounded, Option<&Player>), With<AffectedByGravity>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let dt = time.delta_secs();
    for (mut vel, grounded, player) in query.iter_mut() {
        if grounded.0 {
            // Only reset downward velocity on ground, keep upward for jumps
            if vel.y < 0.0 {
                vel.y = 0.0;
            }
            continue;
        }
        let gravity = if player.is_some() && keys.pressed(KeyCode::KeyZ) {
            GRAVITY_HOLD
        } else {
            GRAVITY_RELEASE
        };
        vel.y -= gravity * dt;
        if vel.y < MAX_FALL {
            vel.y = MAX_FALL;
        }
    }
}

// Collects static AABBs into (min, max) pairs
#[derive(Clone, Copy)]
struct Aabb {
    min: Vec2,
    max: Vec2,
}

impl Aabb {
    fn new(center: Vec2, size: Vec2) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

pub fn move_entities(
    time: Res<Time>,
    static_q: Query<(Entity, &Transform, &BoxCollider), (With<Solid>, Without<Velocity>)>,
    moving_platform_q: Query<
        (Entity, &Transform, &BoxCollider),
        (With<Solid>, With<Velocity>, Without<AffectedByGravity>),
    >,
    mut dynamic_q: Query<
        (&mut Transform, &mut Velocity, &BoxCollider, &mut Grounded, Option<&Player>),
        With<AffectedByGravity>,
    >,
    mut bump_events: EventWriter<BumpEvent>,
) {
    let dt = time.delta_secs();

    // Collect all solid AABBs with their entity IDs
    let mut solids: Vec<(Aabb, Entity)> = static_q
        .iter()
        .map(|(e, t, c)| (Aabb::new(t.translation.truncate(), c.size), e))
        .collect();
    for (e, t, c) in moving_platform_q.iter() {
        solids.push((Aabb::new(t.translation.truncate(), c.size), e));
    }

    for (mut transform, mut vel, coll, mut grounded, player) in dynamic_q.iter_mut() {
        let half = coll.size * 0.5;
        grounded.0 = false;

        // ── Move X ──────────────────────────────────────────────────
        transform.translation.x += vel.x * dt;
        let pos = transform.translation.truncate();
        let me = Aabb::new(pos, coll.size);
        for (solid, _) in &solids {
            if me.overlaps(solid) {
                if vel.x > 0.0 {
                    transform.translation.x = solid.min.x - half.x;
                    vel.x = 0.0;
                } else if vel.x < 0.0 {
                    transform.translation.x = solid.max.x + half.x;
                    vel.x = 0.0;
                }
            }
        }

        // ── Move Y ──────────────────────────────────────────────────
        transform.translation.y += vel.y * dt;
        let pos = transform.translation.truncate();
        let me = Aabb::new(pos, coll.size);
        for (solid, solid_entity) in &solids {
            if me.overlaps(solid) {
                if vel.y < 0.0 {
                    // landing on top of a solid
                    transform.translation.y = solid.max.y + half.y;
                    vel.y = 0.0;
                    grounded.0 = true;
                } else if vel.y > 0.0 {
                    // bumped ceiling
                    transform.translation.y = solid.min.y - half.y;
                    vel.y = 0.0;
                    if player.is_some() {
                        bump_events.send(BumpEvent(*solid_entity));
                    }
                }
            }
        }
    }
}
