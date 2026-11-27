# xrf-game-mcp

The native game's control endpoint, for agents to drive, debug and view the game. Two halves over one line protocol:

- **The game's side** (`GameEndpoint`), which the game takes behind its `mcp` feature and starts only for `--mcp`:
  a thread reads request lines from the game's standard input into a channel the game drains between frames, and
  answers go back a line each on its standard output. Without `--mcp` the game starts no reader and never reads its
  input.
- **The bridge** (the `xrf-game-mcp` executable), an MCP server over its own standard streams: it starts the game built
  beside it as its child and speaks the line protocol over the child's. It is its own executable, never a subcommand
  of the game's, so an agent session holding it open never keeps `xrf-game.exe` from being built again.

## The line protocol

Each line is one JSON object (`wire`):

- A request (`GameRequest`): an id and its `kind`, with the kind's arguments beside them:
  `{"id":"3","kind":"pose","position":[1,2,3],"target":[1,2,4]}`. Kinds (`GameRequestKind`): `status`, `pose`,
  `input`, `capture`, `quit`.
- `input` carries steps (`GameInput`, each named by its `type`): `press`, `down` and `up` a key by SDL's scancode name
  (`W`, `Grave`, `Return`, `Left Shift`), `text`, `move`, `click`, `wheel`, and `wait`. The game delivers them to its
  play loop as the window's own events, after any played before, and answers once the last has run out, its last wait
  included. Waits total 20 s at most, under the bridge's 30 s wait for an answer.
- An answer (`GameResponse`): the request's id, `ok`, and a `result` or an `error`.
- An event the game sends unasked (`GameEvent`): `{"event":"ready","level":"zaton"}` once the renderer reports the
  level ready, every sector, particle and texture settled, so a capture after it shows the whole level.

Rules:

- The game's input ending quits it, so a bridge that dies leaves no game holding its executable.
- Each side skips a line it cannot read.
- The game's standard error is not piped: its log file holds every record.
- A capture is answered as the path of the PNG the game wrote.

## The bridge's tools

`game_start` (a tree's `root` and a `level`, more arguments optional; waits until the game says it is ready),
`game_status`, `game_pose`, `game_input`, `game_capture`, `game_log` and `game_quit`. `game_log` reads the newest
`xrf-game.*.log` from disk (`GameLogs`), so a game that hung or crashed still has its log read. The input ending quits
a game still running. It answers `initialize`, `ping`, `tools/list` and `tools/call` as the engine's own server does
(`xrf-engine/cli/mcp/McpStdioServer.ts`), with no async runtime.

## Checks

Run `cargo test --locked -p xrf-game-mcp` from the repository root.
