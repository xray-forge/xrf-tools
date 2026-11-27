# xrf-game-loop

The game loop: the game's state stepped at a fixed rate on a thread of its own, and what presentation reads of it. It
knows no gameplay: the game puts its systems together into one `LoopState`, and the loop steps that.

## The step

`LoopStepper` holds the state and its `LoopTime` (the tick, the step, the game day's clock, game seconds elapsed).
`step` gives the state the commands that arrived since the last tick, in the order they arrived, then steps it from the
tick's time. It has no clock of its own: the same commands step the same state to the same frames, which a recorded
session checks (`src/tests/loop_stepper.rs`).

- `LoopState` is the composition root: the whole state as the game puts its systems together, whose `Command` is every
  change from outside (the player, the console, an agent). It routes each command to its system, steps its systems in
  its order, and publishes a `Frame` presentation reads.
- `LoopSystem` is one part of the state and its rules (movement now; the objects and ALife later). It owns its state and
  reaches nothing else by itself: what it needs of the rest (the registry's tables, another system's values, the hook
  bus) is its `Context`, which the root hands it each step (`()` for none). Dependencies are injected as typed
  parameters, never looked up from a container, so they show in the signature and the borrow checker proves two
  systems never write the same data; a system is tested by building its context alone (`src/tests/test_state.rs`).
- The game's day runs by `GameClock`, `[alife]`'s `start_time` and `time_factor` (Call of Pripyat's 9:00 and 10 by
  default): game seconds pass by ticks, never by the wall clock.

## The thread

`GameLoop::start` steps a `LoopStepper` by a `LoopClock`: tick `k` stands at `anchor + k × step`, 100 ticks a second
by default (X-Ray's physics step). It sleeps to each tick's instant with `thread::sleep` rather than a timed receive,
whose wait on Windows is coarser, takes the commands waiting, steps and publishes. Behind its clock by more than 25 ticks, the clock gives that time up rather than stepping
through it at once. Dropping the loop stops its thread and waits for it.

## What presentation reads

`LoopFeed` keeps the last two published `LoopFrame`s: the tick, the time of day and its factor, and the state's own
frame, which blends by `LoopBlend`. A frame is stamped with its tick's instant, not with when it was stepped.
`read(now)` answers the state one tick behind the latest, blended between the two, so motion stays even however late a
step ran; the lock is held only to copy two frames.

## Checks

Run `cargo test --locked -p xrf-game-loop` from the repository root.
