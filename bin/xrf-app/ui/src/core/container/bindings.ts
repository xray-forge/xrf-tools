import { Binding } from "@wirestate/core";
import {
  IpcMetricsService,
  SESSION_TOOL,
  SessionChangesService,
  SessionScopeService,
  SHELL_SESSION_TOOL,
} from "@xrf/ipc/session";

import { AudioSettingsService } from "@/core/audio/services/audio-settings.service";
import { KeybindCommandsService } from "@/core/commands";
import { ROOT_KEYBIND_COMMANDS } from "@/core/container/root-commands";
import { HelpService } from "@/core/help/services/help.service";
import { JobsService } from "@/core/jobs/services/jobs.service";
import { KEYMAP_ROOT_COMMANDS } from "@/core/keybinds/lib/keymap-root-commands";
import { KeymapService } from "@/core/keybinds/services/keymap.service";
import { LAUNCHER_KEYBIND_COMMANDS } from "@/core/launcher/commands";
import { LAUNCHER_HELP } from "@/core/launcher/help";
import { ErrorCaptureService } from "@/core/notifications/services/error-capture";
import { NotificationsService } from "@/core/notifications/services/notifications";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { LAUNCHER_SCREEN } from "@/core/routing/launcher-screen";
import { ShellToolService } from "@/core/routing/services/shell-tool.service";
import { SettingsService } from "@/core/settings/services/settings.service";
import { SettingsAboutService } from "@/core/settings-dialog/services/settings-about";
import { SettingsDialogService } from "@/core/settings-dialog/services/settings-dialog";
import { SettingsStorageService } from "@/core/settings-dialog/services/settings-storage";
import { ShellContributionsService } from "@/core/shell/services/shell-contributions";
import { ShellKeybindsService } from "@/core/shell/services/shell-keybinds";
import { ToolLifecycleService } from "@/core/shell/services/tool-lifecycle";

/**
 * The services the root container binds, which every tool resolves through. The shell's own openings are held
 * in its scope; each tool's container binds a scope of its own over it.
 */
export const ROOT_BINDINGS: ReadonlyArray<Binding> = [
  { token: SESSION_TOOL, value: SHELL_SESSION_TOOL },
  { token: KEYMAP_ROOT_COMMANDS, value: ROOT_KEYBIND_COMMANDS },
  { token: LAUNCHER_SCREEN, value: { help: LAUNCHER_HELP, keybindCommands: LAUNCHER_KEYBIND_COMMANDS } },
  SessionScopeService,
  AudioSettingsService,
  ErrorCaptureService,
  HelpService,
  IpcMetricsService,
  JobsService,
  KeybindCommandsService,
  KeymapService,
  NotificationsService,
  RenderSettingsService,
  SessionChangesService,
  SettingsAboutService,
  SettingsDialogService,
  SettingsService,
  SettingsStorageService,
  ShellContributionsService,
  ShellKeybindsService,
  ShellToolService,
  ToolLifecycleService,
];
