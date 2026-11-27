# xrf-game-console

The game's console: the commands and variables a line typed runs, as X-Ray's console runs them. The developer layer's
drawer and the bridge's `game_console` tool enter lines through the same registry, so an agent does what a player
types.

- `ConsoleRegistry<F>` holds the commands (`ConsoleCommand::new`: a name, how it is written, what it does, and the
  closure that runs it, `ConsoleRun`) over the contexts of a family `F` (`ConsoleFamily`: `Context<'a>` is the
  program's state borrowed for `'a`, so the program lends its console what it owns for one line at a time and the
  registry keeps none; a context that owns its variables is its own family, `type Context<'a> = Self`). A command is
  higher-ranked over the borrow, so its closure names the context's type in full (`|state: &mut GameConsoleState<'_>,
  _|`); one that leaves it unnamed does not infer. `register` is its hook point: a system, and later a script, adds a
  command there, the closure carrying what it needs, and one registered under a name already taken, `help` and `list`
  included, overrides it. `execute(context, line)` runs a line
  (`ConsoleLine`: a name, then what follows it): a command by its name; a variable's name alone shows it, with a value
  sets it; `help` lists the commands and `list [part]` the variables. An unknown name fails, suggesting variables
  whose name holds it. `complete(context, line)` gives what a line may become (`ConsoleCompletion`): the name typed
  completed as far as every name it may still be shares, a space after one that is whole, as Tab completes it, and the
  choices left (`ConsoleChoice`), each to its next dot with a hint, as a suggestion list shows them: `view.s` offers
  `view.shadows.` (16 variables), `view.sun_shafts.` and `view.sun_tint` (= 0.0).
- `ConsoleContext` is what the context gives the registry: its variables (`ConsoleVariable`, a name and a value as
  text) and how one is set. The game's variables are its settings, by their dotted paths.
- `ConsoleAnswer` is what a line answers: the lines shown, and whether it failed, which the bridge reports as an error.

It depends on nothing of the game: the game builds its registry and lends it its context, the settings store play
owns and where a line's asks go, for each line. The game's own commands
(`GameConsoleCommands`) change another system only by sending its `GameCommand` through the game loop, so the console
knows no system.

## Checks

Run `cargo test --locked -p xrf-game-console` from the repository root.
