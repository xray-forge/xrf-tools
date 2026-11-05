import { MyLocationIcon } from "@xrf/ui/icons/my-location";

import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

/**
 * Opens the field the camera readout turns into, which stands the camera wherever coordinates typed or pasted into it
 * say; `mod+g`, as an editor goes to a line.
 */
export const GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["mod+g"],
  description:
    "Stands the camera where typed or pasted coordinates say: the readout's `x 0 y 0 z 0 h 0 p 0`, part of it, or " +
    "bare numbers in that order, as a config writes a position.",
  icon: <MyLocationIcon />,
  id: "level/go-to",
  label: "Go to coordinates…",
});

/** Drops what is selected in the level, while the scene has the keyboard. */
export const CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["Escape"],
  description: "Drops what is selected in the level, while the scene has the keyboard.",
  id: "level/clear-selection",
  label: "Clear selection",
});

/** Stands the camera framing the selected level objects. */
export const FRAME_LEVEL_SELECTION_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["f"],
  description: "Stands the camera framing the selected level objects.",
  id: "level/frame-selection",
  label: "Frame selection",
});

/** Hides the selected level objects until revealed or another level opens. */
export const HIDE_LEVEL_SELECTION_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["h"],
  description: "Hides the selected level objects until they are revealed or another level opens.",
  id: "level/hide-selection",
  label: "Hide selection",
});

/** Hides every level object but the selected. */
export const ISOLATE_LEVEL_SELECTION_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["shift+h"],
  description: "Hides every level object but the selected.",
  id: "level/isolate-selection",
  label: "Isolate selection",
});

/** Shows every level object hidden again. */
export const REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["alt+h"],
  description: "Shows every hidden level object again.",
  id: "level/reveal-objects",
  label: "Reveal hidden objects",
});

/** Draws the level objects' helpers or leaves them out, each kind keeping its choice: the inspector and photo modes. */
export const TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["g"],
  description: "Draws the level objects' helpers or leaves them all out, each kind keeping its choice.",
  id: "level/toggle-helpers",
  label: "Toggle helpers",
});

/** Commands the level domain owns, reachable from every tool that shows a level. */
export const LEVEL_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [
  GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND,
  CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND,
  FRAME_LEVEL_SELECTION_KEYBIND_COMMAND,
  TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND,
  HIDE_LEVEL_SELECTION_KEYBIND_COMMAND,
  ISOLATE_LEVEL_SELECTION_KEYBIND_COMMAND,
  REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND,
];
