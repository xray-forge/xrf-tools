# xrf-random

Random numbers a simulation can repeat: the same seed gives the same numbers on every machine, in every run, on any
number of threads, whatever order its parts draw in.

`RandomStream::new(seed, address)` is the stream at a `RandomAddress` (what the numbers decide, for which thing, at
which step) under a seed. Its draws (`next_u64`, `next_u32`, `next_f32` and `next_f64` from 0 up to 1, `below` an
unbiased whole number, `chance`) are integer arithmetic, and a float a scaled integer, exact everywhere. Nothing is kept
between streams, so a save keeps only the seed, and a stream is made where it is used:

```rust
use xrf_random::{RandomAddress, RandomStream};

const MOVEMENT: u64 = RandomAddress::domain("alife.movement");

let mut stream = RandomStream::new(0x5eed, RandomAddress { domain: MOVEMENT, entity: 412, step: 9_120 });
let turn: u64 = stream.below(3);
```

## Why counter-based

A generator with state (xoshiro, PCG, a seeded `rand` generator) hands out numbers in the order it is asked: two
systems sharing one, or one system on several threads, change each other's numbers by when they ask, and a save must
keep every generator's state. Forking a generator per object (Bevy's `bevy_rand`) fixes the threads but keeps a state
per object, and the forks depend on the order they were made.

A counter-based generator is a keyed function of a counter, not a sequence: the counter is the address, the key the
seed, so any draw is reached directly and none depends on another. It is how parallel numerics repeat themselves: JAX
(Threefry, `fold_in`), cuRAND, NumPy (`Philox`), TensorFlow and PyTorch (Philox); C++26 adds `std::philox_engine`.

`Philox4x64` is Philox4x64-10 (Salmon et al., SC11; the Random123 library), the variant C++26 names `philox4x64`: ten
rounds of 64-bit multiplies, crush-resistant (it passes TestU01's BigCrush), a few nanoseconds a block of four words,
no dependency. It is checked against Random123's known-answer vectors, and the stream at the zero address against C++26's
own check of `philox4x64`. It is no cryptographic generator: nothing here keeps a secret.

## Checks

Run `cargo test --locked -p xrf-random` from the repository root.
