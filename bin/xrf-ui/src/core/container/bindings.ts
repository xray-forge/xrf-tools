import { Binding } from "@wirestate/core";

import { KeybindCommandsService } from "@/core/commands";
import { HelpService } from "@/core/help/services/help";
import { IpcMetricsService } from "@/core/ipc/services/metrics";
import { JobsService } from "@/core/jobs/services/jobs";
import { KeymapService } from "@/core/keybinds/services/keymap";
import { ErrorCaptureService } from "@/core/notifications/services/error-capture.service";
import { NotificationsService } from "@/core/notifications/services/notifications.service";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorLifecycleService } from "@/core/shell/services/editor-lifecycle";
import { EditorShellService } from "@/core/shell/services/editor-shell";

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
  SettingsService,
];
