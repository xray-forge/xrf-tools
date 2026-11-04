import { FiberManualRecordIcon as RecordIcon } from "@xrf/ui/icons/fiber-manual-record";
import { MapOutlinedIcon as MapIcon } from "@xrf/ui/icons/map-outlined";
import { PanoramaPhotosphereOutlinedIcon as PanoramaPhotosphereIcon } from "@xrf/ui/icons/panorama-photosphere-outlined";
import { PhotoCameraOutlinedIcon as PhotoCameraIcon } from "@xrf/ui/icons/photo-camera-outlined";

import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";
import { LEVEL_KEYBIND_COMMANDS } from "@/core/level/commands";

/** Saves the viewport's finished frame, without its overlays, to the screenshots folder; helpers only where asked. */
export const TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["F9"],
  description:
    "Saves the viewport's finished frame, without its overlays, to the screenshots folder; with the helpers where " +
    "Helpers › Include in screenshots is on.",
  icon: <PhotoCameraIcon />,
  id: "level-viewer/take-screenshot",
  label: "Take screenshot",
});

/** Opens the capture on a level map, as `demo_record`'s F11 takes one. */
export const CAPTURE_LEVEL_MAP_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["F11"],
  description: "Opens the level map capture: the level seen straight down, saved to the screenshots folder.",
  icon: <MapIcon />,
  id: "level-viewer/capture-map",
  label: "Capture map…",
});

/**
 * Opens the capture on a cubemap, as `demo_record`'s Backspace takes one; unbound, since Backspace is too easy to press
 * in a field.
 */
export const CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  description: "Opens the cubemap capture: the six faces around the camera, saved to the screenshots folder.",
  icon: <PanoramaPhotosphereIcon />,
  id: "level-viewer/capture-cubemap",
  label: "Capture cubemap…",
});

/**
 * Starts recording the viewport with its sound to the recordings folder, or stops the recording running; beside the
 * screenshot's F9, as both are taken of what the viewport shows.
 */
export const RECORD_LEVEL_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["shift+F9"],
  description:
    "Starts recording the viewport with what its camera hears to the recordings folder, or stops the recording " +
    "running; the camera is heard whatever window takes the focus meanwhile.",
  icon: <RecordIcon />,
  id: "level-viewer/record",
  label: "Record",
});

/** Commands reachable inside the level viewer, beside the root ones. */
export const LEVEL_VIEWER_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [
  ...LEVEL_KEYBIND_COMMANDS,
  TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND,
  RECORD_LEVEL_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
];
