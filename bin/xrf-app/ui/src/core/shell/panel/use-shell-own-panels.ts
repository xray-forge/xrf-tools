import { useInjection } from "@wirestate/react";
import { SHELL_SESSION_TOOL } from "@xrf/ipc/session";
import { useMemo } from "react";

import { SettingsService } from "@/core/settings/services/settings";
import { useShellContribution } from "@/core/shell/contributions";
import { IEditorPanel } from "@/core/shell/editor-shell";
import { JOBS_PANEL } from "@/core/shell/panel/jobs/jobs-panel";
import { NOTIFICATIONS_PANEL } from "@/core/shell/panel/notifications/notification-panel";

/**
 * Publishes the shell's own panels under its key, drawn in every tool's right stripe below the tool's: the jobs
 * listing in dev mode, then the notification log.
 */
export function useShellOwnPanels(): void {
  const settingsService: SettingsService = useInjection(SettingsService);
  const isDevModeEnabled: boolean = settingsService.isDevModeEnabled;

  // Published rather than merely hidden, so a build without dev mode never mounts a panel that polls the backend.
  const panels: Array<IEditorPanel> = useMemo(
    () => (isDevModeEnabled ? [JOBS_PANEL, NOTIFICATIONS_PANEL] : [NOTIFICATIONS_PANEL]),
    [isDevModeEnabled]
  );

  useShellContribution("panels", SHELL_SESSION_TOOL, panels);
}
