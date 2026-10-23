import { Binding } from "@wirestate/core";

import { KeybindCommandsService } from "@/core/commands";
import { HelpService } from "@/core/help/services/help";
import { SessionChangesService } from "@/core/ipc/services/changes";
import { IpcMetricsService } from "@/core/ipc/services/metrics";
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
 * The services the root container binds, which every application resolves through.
 */
export const ROOT_BINDINGS: ReadonlyArray<Binding> = [
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
