//! 0003, arm A -- the health slice in idiomatic Bevy, using observers.
//!
//! Two observers watch one event. One applies a heal, clamped to the current
//! maximum; the other raises the maximum. Both are correct in isolation. Run
//! with `cargo run -p adr-research --bin 0003-health-shield-bevy`.

use bevy::prelude::*;

#[derive(Resource, Debug)]
struct Player {
    health: u32,
    max_health: u32,
}

#[derive(Resource, Default)]
struct FractionLog(Vec<f32>);

#[derive(Event)]
struct TurnResolved {
    heal: u32,
    extra_max: u32,
}

/// Each observer recomputes the derived value the UI would show, because
/// nothing in Bevy knows it is derived from two inputs at once.
fn apply_heal(event: On<TurnResolved>, mut p: ResMut<Player>, mut log: ResMut<FractionLog>) {
    p.health = (p.health + event.heal).min(p.max_health);
    log.0.push(p.health as f32 / p.max_health as f32);
}

fn apply_level_up(event: On<TurnResolved>, mut p: ResMut<Player>, mut log: ResMut<FractionLog>) {
    p.max_health += event.extra_max;
    log.0.push(p.health as f32 / p.max_health as f32);
}

/// `order` decides which observer is registered first. Bevy states that the
/// relative order of observers watching the same event is arbitrary, so this is
/// not a knob a real program would have -- it is here to show that the knob
/// exists and that it changes the answer.
fn run(order: &str) -> (u32, u32, Vec<f32>) {
    let mut world = World::new();
    world.insert_resource(Player {
        health: 60,
        max_health: 100,
    });
    world.init_resource::<FractionLog>();

    if order == "heal first" {
        world.add_observer(apply_heal);
        world.add_observer(apply_level_up);
    } else {
        world.add_observer(apply_level_up);
        world.add_observer(apply_heal);
    }

    // One instant: the player is healed 100 and levels up for +100 max.
    world.trigger(TurnResolved {
        heal: 100,
        extra_max: 100,
    });

    let p = world.resource::<Player>();
    let log = world.resource::<FractionLog>().0.clone();
    (p.health, p.max_health, log)
}

fn main() {
    println!("arm A -- bevy observers");
    println!("start: health 60 / max 100");
    println!("instant: heal 100, level up +100 max\n");

    for order in ["heal first", "level up first"] {
        let (health, max, log) = run(order);
        println!("registered {order:<15} -> health {health:>3} / max {max:>3}");
        println!(
            "{:17}   fraction fired {} time(s): {:?}",
            "",
            log.len(),
            log
        );
    }
}
