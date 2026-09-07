import { Binding } from "@wirestate/core";

import { JobsService } from "@/core/jobs/services/jobs";
import { ErrorCaptureService } from "@/core/notifications/services/error-capture.service";
import { NotificationsService } from "@/core/notifications/services/notifications.service";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorLifecycleService } from "@/core/shell/services/editor-lifecycle";

/**
 * The services the root container binds, which every application resolves through.
 */
export const ROOT_BINDINGS: ReadonlyArray<Binding> = [
  SettingsService,
  NotificationsService,
  ErrorCaptureService,
  JobsService,
  EditorLifecycleService,
];
