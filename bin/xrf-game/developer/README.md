# xrf-game-developer

The game's developer layer: an [egui](https://github.com/emilk/egui) interface drawn over the game in development
builds. It shows a readout in the window's bottom right corner always (the frame rate and time, the drawn size and its
antialiasing, what scales the scene where it renders at another size, and the game's tick), and the console as a
drawer from the top while it is open, as X-Ray's, id Tech's and Unreal's consoles open.

The layer uses egui's core alone, with no integration crate. The game feeds it the platform's events and builds it on
the main thread; `DeveloperLayer::build` returns a `DeveloperFrame`, egui's tessellated meshes and texture changes in
the renderer's own `RenderLayer` types, which the renderer's layer pass draws at the end of the window's pass.

- The console key (the key left of 1, scancode 53) opens and closes the console. Open, the layer takes the keyboard
  and mouse from the game, and the game frees the cursor and accepts text.
- The drawer runs nothing itself: a line entered is handed to the game (`DeveloperConsole::take_entered`), which runs
  it through `xrf-game-console` and pushes the answer and the log's records back as lines; it keeps the last 200.
  Tab hands the line to the game to complete (`take_completing`, answered by `complete`), the cursor left at its end;
  Up and Down walk through the last 100 lines entered.
- Mouse positions arrive in the window's coordinates and are turned into egui's points with the window's pixel density
  and scale.
- Texture changes must reach the renderer before the layer that uses them, and none may be skipped.
