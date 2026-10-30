import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

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
  CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND,
  FRAME_LEVEL_SELECTION_KEYBIND_COMMAND,
  TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND,
  HIDE_LEVEL_SELECTION_KEYBIND_COMMAND,
  ISOLATE_LEVEL_SELECTION_KEYBIND_COMMAND,
  REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND,
];
