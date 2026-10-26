import { defineMenu, IMenu } from "@/core/shell/menu/lib";

/** The window's first menu: what the open tool adds, then settings, then closing. */
export const FILE_MENU: IMenu<"settings" | "close"> = defineMenu({
  id: "shell/file",
  label: "File",
  sections: ["settings", "close"],
  extensionsBefore: "settings",
});

/** The window's last menu: help for what is open, what the open tool adds, then where the project lives and what it is. */
export const HELP_MENU: IMenu<"help" | "about"> = defineMenu({
  id: "shell/help",
  label: "Help",
  sections: ["help", "about"],
  extensionsBefore: "about",
});
