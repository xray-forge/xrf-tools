import { Binding } from "@wirestate/core";
import {
  IpcMetricsService,
  SESSION_TOOL,
  SessionChangesService,
  SessionScopeService,
  SHELL_SESSION_TOOL,
} from "@xrf/ipc/session";

import { KeybindCommandsService } from "@/core/commands";
import { HelpService } from "@/core/help/services/help";
import { JobsService } from "@/core/jobs/services/jobs";
import { KeymapService } from "@/core/keybinds/services/keymap";
import { ErrorCaptureService } from "@/core/notifications/services/error-capture.service";
import { NotificationsService } from "@/core/notifications/services/notifications.service";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { SettingsService } from "@/core/settings/services/settings";
import { SettingsDialogService } from "@/core/settings/services/settings-dialog";
import { EditorLifecycleService } from "@/core/shell/services/editor-lifecycle";
import { EditorShellService } from "@/core/shell/services/editor-shell";
import { ShellKeybindsService } from "@/core/shell/services/shell-keybinds";

/**
 * The services the root container binds, which every application resolves through. The shell's own openings are held
 * in its scope; each application's container binds a scope of its own over it.
 */
export const ROOT_BINDINGS: ReadonlyArray<Binding> = [
  { token: SESSION_TOOL, value: SHELL_SESSION_TOOL },
  SessionScopeService,
  EditorLifecycleService,
  EditorShellService,
  ErrorCaptureService,
  HelpService,
  IpcMetricsService,
  JobsService,
  KeybindCommandsService,
  KeymapService,
  NotificationsService,
  RenderSettingsService,
  SessionChangesService,
  SettingsDialogService,
  SettingsService,
  ShellKeybindsService,
];
