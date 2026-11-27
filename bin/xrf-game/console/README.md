# xrf-game-console

The game's console: the commands and variables a line typed runs, as X-Ray's console runs them. The developer layer's
drawer and the bridge's `game_console` tool enter lines through the same registry, so an agent does what a player
types.

- `ConsoleRegistry<C>` holds the commands (`ConsoleCommand`: a name, how it is written, what it does, and the function
  that runs it) over a context `C`, the program's own state. `execute(context, line)` runs a line
  (`ConsoleLine`: a name, then what follows it): a command by its name; a variable's name alone shows it, with a value
  sets it; `help` lists the commands and `list [part]` the variables. An unknown name fails, suggesting variables
  whose name holds it.
- `ConsoleContext` is what the context gives the registry: its variables (`ConsoleVariable`, a name and a value as
  text) and how one is set. The game's variables are its settings, by their dotted paths.
- `ConsoleAnswer` is what a line answers: the lines shown, and whether it failed, which the bridge reports as an error.

It depends on nothing of the game: the game builds its registry and its context.

## Checks

Run `cargo test --locked -p xrf-game-console` from the repository root.
