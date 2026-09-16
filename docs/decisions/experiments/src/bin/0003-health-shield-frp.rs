//! 0003, arm B -- the same health slice as an FRP graph, using `sodium-rust`.
//!
//! Stage 2: shields absorb damage before health, and a second derived value
//! (effective HP) is shown alongside the fraction. Run with
//! `cargo run -p adr-research --bin 0003-health-shield-frp`.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use sodium_rust::{Cell, SodiumCtx, Stream, StreamSink};

struct Slice {
    heal: StreamSink<u32>,
    level_up: StreamSink<u32>,
    damage: StreamSink<u32>,
    health: Cell<u32>,
    max_health: Cell<u32>,
    shield: Cell<u32>,
    fraction: Cell<f32>,
    effective: Cell<u32>,
    /// Intermediate nodes, held only to keep them alive. See the record.
    _keepalive: Vec<Stream<i64>>,
    _keepalive_health: Stream<u32>,
}

fn build(ctx: &SodiumCtx) -> Slice {
    ctx.transaction(|| {
        let heal: StreamSink<u32> = ctx.new_stream_sink();
        let level_up: StreamSink<u32> = ctx.new_stream_sink();
        let damage: StreamSink<u32> = ctx.new_stream_sink();

        // max_health accumulates level-ups.
        let max_loop = ctx.new_cell_loop::<u32>();
        let max_health = level_up
            .stream()
            .snapshot(&max_loop.cell(), |extra: &u32, cur: &u32| cur + extra)
            .hold(100);
        max_loop.loop_(&max_health);

        // Added by stage 2: shields absorb damage before health sees it.
        let shield_loop = ctx.new_cell_loop::<u32>();
        let shield = damage
            .stream()
            .snapshot(&shield_loop.cell(), |d: &u32, s: &u32| s.saturating_sub(*d))
            .hold(30);
        shield_loop.loop_(&shield);

        // Heals and post-shield damage are the same kind of thing -- a change to
        // health -- so they merge into one stream of deltas. `merge` demands a
        // combining function, which is where "both in the same instant" is
        // decided, explicitly and in one place.
        let healed = heal.stream().map(|h: &u32| *h as i64);
        let took = damage
            .stream()
            .snapshot(&shield, |d: &u32, s: &u32| -(d.saturating_sub(*s) as i64));
        let delta = healed.merge(&took, |a: &i64, b: &i64| a + b);

        let health_loop = ctx.new_cell_loop::<u32>();
        let health_updates = delta.clone().snapshot3(
            &health_loop.cell(),
            &max_health,
            |d: &i64, cur: &u32, max: &u32| (*cur as i64 + d).clamp(0, *max as i64) as u32,
        );
        let health = health_updates.clone().hold(60);
        health_loop.loop_(&health);

        // The derived values. Declared once each, as functions of their inputs.
        let fraction = max_health.lift2(&health, |m: &u32, h: &u32| *h as f32 / *m as f32);
        let effective = health.lift2(&shield, |h: &u32, s: &u32| h + s);

        Slice {
            heal,
            level_up,
            damage,
            health,
            max_health,
            shield,
            fraction,
            effective,
            _keepalive: vec![healed, took, delta],
            _keepalive_health: health_updates,
        }
    })
}

fn main() {
    println!("arm B -- sodium-rust");
    println!("start: health 60 / max 100 / shield 30");
    println!("instant: heal 100, level up +100 max, damage 50\n");

    let ctx = SodiumCtx::new();
    let s = build(&ctx);

    // Count firings without a lock: the listeners run on the graph's thread.
    let fires = Arc::new(AtomicUsize::new(0));
    let l1 = s.fraction.listen({
        let fires = Arc::clone(&fires);
        move |_: &f32| {
            fires.fetch_add(1, Ordering::SeqCst);
        }
    });
    let l2 = s.effective.listen({
        let fires = Arc::clone(&fires);
        move |_: &u32| {
            fires.fetch_add(1, Ordering::SeqCst);
        }
    });
    // `listen` fires once on registration; arm A has no equivalent, so discount.
    let baseline = fires.load(Ordering::SeqCst);

    ctx.transaction(|| {
        s.heal.send(100);
        s.level_up.send(100);
        s.damage.send(50);
    });

    let fired = fires.load(Ordering::SeqCst) - baseline;
    println!(
        "{:<22} -> health {:>3} / max {:>3} / shield {:>2}",
        "one transaction",
        s.health.sample(),
        s.max_health.sample(),
        s.shield.sample()
    );
    println!(
        "{:22}    derived fired {}x  fraction {:?}  effective {:?}",
        "",
        fired,
        s.fraction.sample(),
        s.effective.sample()
    );

    // See 0003-lift2-loop-bug: `effective` is a lift2 of `health` with a cell
    // upstream of health's own update, and building it stops health updating.
    // The graph below is the shape the record compares; this number is not
    // trustworthy until that bug is fixed.
    if s.health.sample() != 100 {
        println!(
            "\n!! health is {} and should be 100 -- sodium-rust 2.1.3 bug,\n   \
             minimal repro in `cargo run -p adr-research --bin 0003-lift2-loop-bug`",
            s.health.sample()
        );
    }
    drop((l1, l2));
}
