# xrf-game-config

The one way the game's systems read the tree's configs (plan decision 25): a typed reader over each section, every
value of its kind or an issue naming where it failed, never a silent fallback. A system exposes `read` over a
`ConfigReader` or a `ConfigFile` and imports neither `xrf-ltx` nor a parser of its own; a condlist is read here too
(`Condlist` and its parts are this crate's value types).

- `ConfigSource` is where configs come from, kept for the game's start: `system.ltx`, resolved once, and any other
  config by its path under `configs`. The game's `GameConfigs` is the tree's, in its dialect; a test holds text.
- `ConfigFile` is one config, a resolution shared as its project keeps it or text read plainly (an object's custom
  data). `read_section` reads a section where it is there, `require_section` says an issue where the game needs it.
- `ConfigReader` reads one section's keys as `ConfigValue`s: numbers (`f32`, `u32`), flags (`on`, `yes`, `true`,
  `1` and their opposites), text, three-number vectors and condlists; a list of names (`read_names`, as
  `utils.parse_names`), the keys in the engine's order (`get_keys`, sorted as `CInifile` keeps them), every key as one
  kind (`read_all`).
- `ConfigIssues` collects every `ConfigIssue` (`file [section] key: message`) a read found, in order; the game logs
  them. A read never stops at one.

## The default policy

X-Ray's own. A key X-Ray reads only where it is there (`READ_IF_EXISTS`, `line_exist`) is read with `read` or
`read_or`: missing, it takes its default silently. A key X-Ray reads unconditioned (`r_float` and the rest, which abort
where it is missing) is read with `require` or `require_or`: missing, it is an issue. A key there but not of its kind
is an issue either way. Where X-Ray aborts, the game says so and plays on with the default the reader names, which
its owner documents as the game it comes from (Call of Pripyat's or Anomaly's).

## Checks

Run `cargo test --locked -p xrf-game-config` from the repository root.
