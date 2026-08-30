import { Binding } from "@wirestate/core";

import { JobsService } from "@/core/jobs/services/jobs";
import { ErrorCaptureService } from "@/core/notifications/services/error-capture.service";
import { NotificationsService } from "@/core/notifications/services/notifications.service";
import { PathsService } from "@/core/settings/services/paths";
import { SettingsService } from "@/core/settings/services/settings";

/**
 * The services the root container binds, which every application resolves through.
 */
export const ROOT_BINDINGS: ReadonlyArray<Binding> = [
  PathsService,
  SettingsService,
  NotificationsService,
  ErrorCaptureService,
  JobsService,
];
