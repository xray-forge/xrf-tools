use std::thread;

use crate::stream::random_address::RandomAddress;
use crate::stream::random_stream::RandomStream;

/// C++26's check of `std::philox4x64` (`[rand.predef]`, LWG 4134): the 10000th word of the engine seeded with its
/// default seed, which this stream is at the zero address.
#[test]
fn draws_cpp26s_philox4x64_at_the_zero_address() {
  let mut stream: RandomStream = RandomStream::new(20_111_115, RandomAddress::default());

  let tenth_thousandth: u64 = (0..10_000).map(|_| stream.next_u64()).last().unwrap_or_default();

  assert_eq!(tenth_thousandth, 3_409_172_418_970_261_260);
}

#[test]
fn draws_the_same_at_an_address_whoever_draws_and_wherever() {
  let address = |entity: u64| RandomAddress {
    domain: RandomAddress::domain("test"),
    entity,
    step: 7,
  };
  let draw = |entity: u64| -> Vec<u64> {
    let mut stream: RandomStream = RandomStream::new(42, address(entity));

    (0..9).map(|_| stream.next_u64()).collect()
  };

  // Drawn in order here, and in reverse on threads of their own.
  let in_order: Vec<Vec<u64>> = (0..8).map(draw).collect();
  let mut on_threads: Vec<Vec<u64>> = thread::scope(|scope| {
    let handles: Vec<_> = (0..8).rev().map(|entity| scope.spawn(move || draw(entity))).collect();

    handles
      .into_iter()
      .map(|handle| handle.join().unwrap_or_default())
      .collect()
  });

  on_threads.reverse();

  assert_eq!(in_order, on_threads);
  assert_ne!(in_order[0], in_order[1]);
}

#[test]
fn differs_in_every_part_of_its_address_and_its_seed() {
  let base: RandomAddress = RandomAddress {
    domain: 1,
    entity: 2,
    step: 3,
  };
  let first = |seed: u64, address: RandomAddress| RandomStream::new(seed, address).next_u64();
  let words: Vec<u64> = vec![
    first(9, base),
    first(10, base),
    first(9, RandomAddress { domain: 4, ..base }),
    first(9, RandomAddress { entity: 4, ..base }),
    first(9, RandomAddress { step: 4, ..base }),
  ];

  for (index, word) in words.iter().enumerate() {
    assert!(!words[index + 1..].contains(word));
  }
}

#[test]
fn keeps_numbers_in_their_ranges_and_spreads_them_evenly() {
  let mut stream: RandomStream = RandomStream::new(1, RandomAddress::default());
  let mut counts: [u32; 6] = [0; 6];

  for _ in 0..60_000 {
    let unit: f32 = stream.next_f32();
    let wide: f64 = stream.next_f64();

    assert!((0.0..1.0).contains(&unit) && (0.0..1.0).contains(&wide));

    counts[usize::try_from(stream.below(6)).unwrap_or_default()] += 1;
  }

  // Each face of a die about 10,000 times: well inside five standard deviations (about 460).
  assert!(counts.iter().all(|count| count.abs_diff(10_000) < 460), "{counts:?}");
  assert_eq!(stream.below(0), 0);
  assert_eq!(stream.below(1), 0);
}

#[test]
fn comes_true_by_its_chance() {
  let mut stream: RandomStream = RandomStream::new(3, RandomAddress::default());

  assert!((0..1_000).all(|_| !stream.chance(0.0)));
  assert!((0..1_000).all(|_| stream.chance(1.0)));

  let hits: usize = (0..10_000).filter(|_| stream.chance(0.25)).count();

  assert!(hits.abs_diff(2_500) < 220, "{hits}");
}
