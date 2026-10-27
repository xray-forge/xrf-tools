import { Window } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-shell";
import { useInjection } from "@wirestate/react";
import { SHELL_SESSION_TOOL } from "@xrf/ipc/session";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";
import { GitHubIcon } from "@xrf/ui/icons/git-hub";
import { InfoOutlinedIcon } from "@xrf/ui/icons/info-outlined";
import { useMemo, useState } from "react";

import { REPOSITORY_URL } from "@/core/configs";
import { OPEN_TOOL_HELP_KEYBIND_COMMAND } from "@/core/help/commands";
import { OPEN_SETTINGS_KEYBIND_COMMAND } from "@/core/settings/commands";
import { ESettingsSection } from "@/core/settings/lib/settings-section";
import { SettingsDialogService } from "@/core/settings/services/settings-dialog";
import { CLOSE_TOOL_KEYBIND_COMMAND } from "@/core/shell/commands";
import { useShellContribution } from "@/core/shell/contributions";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { FILE_MENU, HELP_MENU } from "@/core/shell/menu/menus";
import { resolveAppWindow } from "@/lib/tauri/app-window";

function onOpenRepository(): void {
  open(REPOSITORY_URL).catch(Logger.error);
}

/** Publishes the shell's own File and Help entries under its key, beside whatever the open tool adds. */
export function useShellOwnMenus(): void {
  const settingsDialogService: SettingsDialogService = useInjection(SettingsDialogService);
  const [appWindow] = useState<Nullable<Window>>(resolveAppWindow);

  const contributions: Array<IMenuContribution> = useMemo(
    () => [
      {
        section: FILE_MENU.sections.settings,
        entries: [{ kind: "command", command: OPEN_SETTINGS_KEYBIND_COMMAND }],
      },
      {
        section: FILE_MENU.sections.close,
        entries: [
          { kind: "command", command: CLOSE_TOOL_KEYBIND_COMMAND },
          {
            kind: "action",
            id: "shell/exit",
            label: "Exit",
            isDisabled: !appWindow,
            // Closing asks the window to close, which the page's close guard then answers like the caption's close
            // button.
            onSelect: () => appWindow?.close().catch(Logger.error),
          },
        ],
      },
      {
        section: HELP_MENU.sections.help,
        entries: [{ kind: "command", command: OPEN_TOOL_HELP_KEYBIND_COMMAND }],
      },
      {
        section: HELP_MENU.sections.about,
        entries: [
          {
            kind: "action",
            id: "shell/open-repository",
            label: "Source",
            icon: <GitHubIcon />,
            onSelect: onOpenRepository,
          },
          {
            kind: "action",
            id: "settings/open-about",
            label: "About",
            icon: <InfoOutlinedIcon />,
            onSelect: () => settingsDialogService.openSection(ESettingsSection.ABOUT),
          },
        ],
      },
    ],
    [appWindow, settingsDialogService]
  );

  useShellContribution("menus", SHELL_SESSION_TOOL, contributions);
}
