//! 0003, arm A -- the health slice in idiomatic Bevy, using observers.
//!
//! Stage 2: shields absorb damage before health, and a second derived value
//! (effective HP) is shown alongside the fraction. Run with
//! `cargo run -p adr-research --bin 0003-health-shield-bevy`.

use bevy::prelude::*;

#[derive(Resource, Debug)]
struct Player {
    health: u32,
    max_health: u32,
    shield: u32,
}

#[derive(Resource, Default)]
struct DerivedLog {
    fraction: Vec<f32>,
    effective: Vec<u32>,
}

#[derive(Event)]
struct TurnResolved {
    heal: u32,
    extra_max: u32,
    damage: u32,
}

/// Every observer that touches an input has to call this, because nothing in
/// Bevy knows these two values are derived. Adding a third derived value means
/// editing it; adding a fourth observer means remembering to call it.
fn recompute(p: &Player, log: &mut DerivedLog) {
    log.fraction.push(p.health as f32 / p.max_health as f32);
    log.effective.push(p.health + p.shield);
}

fn apply_heal(event: On<TurnResolved>, mut p: ResMut<Player>, mut log: ResMut<DerivedLog>) {
    p.health = (p.health + event.heal).min(p.max_health);
    recompute(&p, &mut log);
}

fn apply_level_up(event: On<TurnResolved>, mut p: ResMut<Player>, mut log: ResMut<DerivedLog>) {
    p.max_health += event.extra_max;
    recompute(&p, &mut log);
}

/// Added by stage 2. Shields absorb first, the remainder reaches health.
fn apply_damage(event: On<TurnResolved>, mut p: ResMut<Player>, mut log: ResMut<DerivedLog>) {
    let absorbed = event.damage.min(p.shield);
    p.shield -= absorbed;
    p.health = p.health.saturating_sub(event.damage - absorbed);
    recompute(&p, &mut log);
}

fn fresh() -> World {
    let mut world = World::new();
    world.insert_resource(Player {
        health: 60,
        max_health: 100,
        shield: 30,
    });
    world.init_resource::<DerivedLog>();
    world
}

fn turn() -> TurnResolved {
    TurnResolved {
        heal: 100,
        extra_max: 100,
        damage: 50,
    }
}

/// Bevy states that the relative order of observers watching the same event is
/// arbitrary. `order` is not a knob a real program has -- it is here to show
/// that the knob exists and that it changes the answer.
fn run_observers(order: &str) -> (Player, DerivedLog) {
    let mut world = fresh();
    if order == "heal first" {
        world.add_observer(apply_heal);
        world.add_observer(apply_level_up);
        world.add_observer(apply_damage);
    } else {
        world.add_observer(apply_damage);
        world.add_observer(apply_level_up);
        world.add_observer(apply_heal);
    }
    world.trigger(turn());
    take(world)
}

/// The steelman. Observers only mutate; the derived values are computed once,
/// afterwards, the way a later system with change detection would. This is the
/// fix a Bevy reviewer would reach for, and it is worth knowing what it does
/// and does not solve.
fn run_derive_after() -> (Player, DerivedLog) {
    let mut world = fresh();
    world.add_observer(|event: On<TurnResolved>, mut p: ResMut<Player>| {
        p.health = (p.health + event.heal).min(p.max_health);
    });
    world.add_observer(|event: On<TurnResolved>, mut p: ResMut<Player>| {
        p.max_health += event.extra_max;
    });
    world.add_observer(|event: On<TurnResolved>, mut p: ResMut<Player>| {
        let absorbed = event.damage.min(p.shield);
        p.shield -= absorbed;
        p.health = p.health.saturating_sub(event.damage - absorbed);
    });
    world.trigger(turn());

    let p = world.resource::<Player>();
    let mut log = DerivedLog::default();
    recompute(p, &mut log);
    let p = Player {
        health: p.health,
        max_health: p.max_health,
        shield: p.shield,
    };
    (p, log)
}

fn take(world: World) -> (Player, DerivedLog) {
    let p = world.resource::<Player>();
    let p = Player {
        health: p.health,
        max_health: p.max_health,
        shield: p.shield,
    };
    let log = world.resource::<DerivedLog>();
    let log = DerivedLog {
        fraction: log.fraction.clone(),
        effective: log.effective.clone(),
    };
    (p, log)
}

fn report(label: &str, p: &Player, log: &DerivedLog) {
    println!(
        "{label:<22} -> health {:>3} / max {:>3} / shield {:>2}",
        p.health, p.max_health, p.shield
    );
    println!(
        "{:22}    derived fired {}x  fraction {:?}  effective {:?}",
        "",
        log.fraction.len(),
        log.fraction,
        log.effective
    );
}

fn main() {
    println!("arm A -- bevy observers");
    println!("start: health 60 / max 100 / shield 30");
    println!("instant: heal 100, level up +100 max, damage 50\n");

    for order in ["heal first", "damage first"] {
        let (p, log) = run_observers(order);
        report(&format!("registered {order}"), &p, &log);
    }
    let (p, log) = run_derive_after();
    report("derive afterwards", &p, &log);
}
