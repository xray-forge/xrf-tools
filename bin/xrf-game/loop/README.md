# xrf-game-loop

The game loop: the game's state stepped at a fixed rate on a thread of its own, and what presentation reads of it. It
knows no gameplay: the game puts its systems together into one `LoopState`, and the loop steps that.

## The step

`LoopStepper` holds the state and its `LoopTime` (the tick and the step). `step` gives the state the commands that
arrived since the last tick, in the order they arrived, then steps it from the tick's time, answering the events the
step told. It has no clock of its own: the same commands step the same state to the same frames, which a recorded
session checks (`src/tests/loop_stepper.rs`).

- `LoopState` is the composition root: the whole state as the game puts its systems together, whose `Command` is every
  change from outside (the player, the console, an agent). It routes each command to its system, steps its systems in
  its order, tells its `Event`s, and publishes a `Frame` presentation reads.
- `LoopSystem` is one part of the state and its rules (movement, A-Life). It owns its state and reaches nothing else by
  itself: what it needs of the rest (the registry's tables, another system's values, where it tells its events, the
  hook registry) is its `Context`, which the root hands it each step (`()` for none). Dependencies are injected as typed
  parameters, never looked up from a container, so they show in the signature and the borrow checker proves two
  systems never write the same data; a system is tested by building its context alone (`src/tests/test_state.rs`).
- A step answers a fault rather than logging it and stepping on half applied: the loop then stops, logs the fault once
  and says so in its status.
- Game time is the game's state, not the loop's: the loop counts ticks and hands each step its length, and the game
  moves its own time by them, never by the wall clock, so a save keeps it and the game may set it.

## The thread

`GameLoop::start` steps a `LoopStepper` by a `LoopClock`: tick `k` stands at `anchor + k × step`, 100 ticks a second
by default (X-Ray's physics step). It sleeps to each tick's instant with `thread::sleep` rather than a timed receive,
whose wait on Windows is coarser, takes the commands waiting, steps, publishes the frame and queues the step's events.
Behind its clock by more than 25 ticks, the clock gives that time up rather than stepping through it at once. `pause`
stops it stepping before the next tick (commands wait meanwhile) and `resume` steps again from a step after now, the
time paused given up; `snapshot` reads the state between two ticks, paused or not, its answer on a channel of its own:
a query, as the control endpoint's status, a level change or a save reads it. Dropping the loop stops its thread and
waits for it.

## What presentation reads

- `LoopFeed` keeps the last two published `LoopFrame`s, shared (`Arc`) rather than copied, so a frame holds tables of
  any size: the tick and the state's own frame. A frame is stamped with its tick's instant, not with when it was
  stepped. `read(now)` answers both frames and how far between them `now` stands, one tick behind the latest
  (`LoopRead`), and presentation blends what it draws of them (`LoopBlend` for a value); the lock is held only to clone
  two handles. The feed also carries `LoopStatus`: running, faulted (the tick and the fault, a panic among them) or
  stopped.
- `LoopEvents` queues what the steps told, each `LoopEvent` stamped with its tick, in order, never dropped or merged
  however rarely presentation takes them, unlike the frames, which it samples.

## Checks

Run `cargo test --locked -p xrf-game-loop` from the repository root.
