import { default as SettingsIcon } from "@mui/icons-material/Settings";

import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

/** Opens the settings dialog where it was left. */
export const OPEN_SETTINGS_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.APPLICATION,
  chords: ["mod+,"],
  description: "Opens the settings dialog at the section it was left on.",
  icon: <SettingsIcon />,
  id: "settings/open",
  label: "Settings",
});

/** Commands settings owns, reachable from every screen. */
export const SETTINGS_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [OPEN_SETTINGS_KEYBIND_COMMAND];
