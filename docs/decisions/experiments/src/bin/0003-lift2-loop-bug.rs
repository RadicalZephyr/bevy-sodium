//! 0003 -- a correctness bug in `sodium-rust` 2.1.3, found while building arm B.
//!
//! `lift2` is `Apply` in the specification (section 5.14): a pure function of
//! two cells. It cannot change either of them. Here it does: adding
//! `health.lift2(&shield, ..)` -- a value nothing else reads -- stops `health`
//! updating at all.
//!
//! The shape that triggers it is a diamond through a loop. `health` is held by
//! a `CellLoop`, and `health`'s own update stream reads `shield` (damage is
//! absorbed by the shield before it reaches health). Lifting `health` together
//! with `shield` is then lifting a cell with something upstream of itself.
//!
//! Run with `cargo run -p adr-research --bin 0003-lift2-loop-bug`.

use sodium_rust::{SodiumCtx, StreamSink};

/// 0 = no lift, 1 = lift2(max_health, health), 2 = lift2(health, shield), 3 = both
fn scenario(which: u8) -> u32 {
    let ctx = SodiumCtx::new();
    let (heal, damage, level_up, health, keep) = ctx.transaction(|| {
        let heal: StreamSink<u32> = ctx.new_stream_sink();
        let damage: StreamSink<u32> = ctx.new_stream_sink();
        let level_up: StreamSink<u32> = ctx.new_stream_sink();

        let max_loop = ctx.new_cell_loop::<u32>();
        let max_health = level_up
            .stream()
            .snapshot(&max_loop.cell(), |e: &u32, c: &u32| c + e)
            .hold(100);
        max_loop.loop_(&max_health);

        let shield_loop = ctx.new_cell_loop::<u32>();
        let shield = damage
            .stream()
            .snapshot(&shield_loop.cell(), |d: &u32, s: &u32| s.saturating_sub(*d))
            .hold(30);
        shield_loop.loop_(&shield);

        let healed = heal.stream().map(|h: &u32| *h as i64);
        let took = damage
            .stream()
            .snapshot(&shield, |d: &u32, s: &u32| -(d.saturating_sub(*s) as i64));
        let delta = healed.merge(&took, |a: &i64, b: &i64| a + b);

        let health_loop = ctx.new_cell_loop::<u32>();
        let health = delta
            .snapshot3(
                &health_loop.cell(),
                &max_health,
                |d: &i64, cur: &u32, max: &u32| (*cur as i64 + d).clamp(0, *max as i64) as u32,
            )
            .hold(60);
        health_loop.loop_(&health);

        let mut keep: Vec<Box<dyn std::any::Any>> = vec![];
        if which == 1 || which == 3 {
            keep.push(Box::new(
                max_health.lift2(&health, |m: &u32, h: &u32| *h as f32 / *m as f32),
            ));
        }
        if which == 2 || which == 3 {
            keep.push(Box::new(health.lift2(&shield, |h: &u32, s: &u32| h + s)));
        }
        (heal, damage, level_up, health, keep)
    });
    ctx.transaction(|| {
        heal.send(100);
        level_up.send(100);
        damage.send(50);
    });
    let h = health.sample();
    drop(keep);
    h
}

fn main() {
    println!("expected health = 100 in every row (lift2 is observation only)\n");
    for (which, label) in [
        (0u8, "no lift2"),
        (1, "lift2(max_health, health)"),
        (2, "lift2(health, shield)"),
        (3, "both"),
    ] {
        let h = scenario(which);
        println!(
            "{label:<28} health = {h:>3}  {}",
            if h == 100 { "ok" } else { "WRONG" }
        );
    }
}
