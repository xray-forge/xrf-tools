# xrf-collections

The maps, sets and queues every XRF crate uses, and the one hasher they all build with. It is the only crate that
depends on `indexmap`, `rustc-hash`, `hashbrown` and `rtrb`; every other crate takes them from here.

| Type               | Is                                                                                  |
| ------------------ | ----------------------------------------------------------------------------------- |
| `OrderedMap<K, V>` | A map keeping insertion order (`indexmap`), as a file lists its sections and fields |
| `OrderedSet<T>`    | A set keeping insertion order                                                       |
| `FastMap<K, V>`    | A map in no particular order (`std`)                                                |
| `FastSet<T>`       | A set in no particular order                                                        |
| `FastBuildHasher`  | The hasher all four build with: `rustc-hash`'s Fx                                   |

```rust
use xrf_collections::OrderedMap;

let mut fields: OrderedMap<&str, &str> = OrderedMap::default();

fields.insert("visual", "dynamics\\weapons\\wpn_ak74");
fields.insert("class", "WP_AK74");

assert_eq!(fields.keys().copied().collect::<Vec<_>>(), ["visual", "class"]);
```

- **One hasher.** Fx is several times faster than SipHash on the short names game files are keyed by. It gives up
  resistance to keys chosen to collide, which nothing here needs: every key comes from a file the user opened or a name
  the tools made.
- **Building one.** `default()`, `with_capacity_and_hasher(n, FastBuildHasher)` or `collect()`; `new` and
  `with_capacity` belong to the standard library's own hasher.
- **The crates themselves** are re-exported for what the aliases do not name: `xrf_collections::indexmap::map::Entry`
  and the iterators, `xrf_collections::hashbrown::HashTable` for an index keyed by hashes it computes, and
  `xrf_collections::rtrb` for handing values to the audio thread without a lock.

## Checks

Run `cargo test --locked -p xrf-collections` from the repository root.
