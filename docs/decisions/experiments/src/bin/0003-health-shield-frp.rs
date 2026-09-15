//! 0003, arm B -- the same health slice as an FRP graph, using `sodium-rust`.
//!
//! Same start state and same instant as arm A. Run with
//! `cargo run -p adr-research --bin 0003-health-shield-frp`.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use sodium_rust::{Cell, SodiumCtx, StreamSink};

struct Slice {
    heal: StreamSink<u32>,
    level_up: StreamSink<u32>,
    health: Cell<u32>,
    max_health: Cell<u32>,
    fraction: Cell<f32>,
}

fn build(ctx: &SodiumCtx) -> Slice {
    ctx.transaction(|| {
        let heal: StreamSink<u32> = ctx.new_stream_sink();
        let level_up: StreamSink<u32> = ctx.new_stream_sink();

        // max_health accumulates level-ups.
        let max_loop = ctx.new_cell_loop::<u32>();
        let max_health = level_up
            .stream()
            .snapshot(&max_loop.cell(), |extra: &u32, cur: &u32| cur + extra)
            .hold(100);
        max_loop.loop_(&max_health);

        // health = clamp(previous + heal, max_health).
        let health_loop = ctx.new_cell_loop::<u32>();
        let health = heal
            .stream()
            .snapshot3(
                &health_loop.cell(),
                &max_health,
                |h: &u32, cur: &u32, max: &u32| (cur + h).min(*max),
            )
            .hold(60);
        health_loop.loop_(&health);

        // The derived value. Declared once, as a function of both inputs.
        let fraction = max_health.lift2(&health, |m: &u32, h: &u32| *h as f32 / *m as f32);

        Slice {
            heal,
            level_up,
            health,
            max_health,
            fraction,
        }
    })
}

fn main() {
    println!("arm B -- sodium-rust");
    println!("start: health 60 / max 100");
    println!("instant: heal 100, level up +100 max\n");

    let ctx = SodiumCtx::new();
    let s = build(&ctx);

    // Count firings without a lock: the listener runs on the graph's thread.
    let fires = Arc::new(AtomicUsize::new(0));
    let listener = s.fraction.listen({
        let fires = Arc::clone(&fires);
        move |_f: &f32| {
            fires.fetch_add(1, Ordering::SeqCst);
        }
    });
    // `listen` fires once on registration with the current value; arm A has no
    // equivalent, so discount it.
    let baseline = fires.load(Ordering::SeqCst);

    // Both sends in one transaction: one instant, as arm A's single trigger was.
    ctx.transaction(|| {
        s.heal.send(100);
        s.level_up.send(100);
    });

    let fired = fires.load(Ordering::SeqCst) - baseline;
    println!(
        "                           -> health {:>3} / max {:>3}",
        s.health.sample(),
        s.max_health.sample()
    );
    println!(
        "{:27}   fraction fired {} time(s), final {:?}",
        "",
        fired,
        s.fraction.sample()
    );
    drop(listener);
}
