import { default as ClearAllIcon } from "@mui/icons-material/ClearAll";
import { IconButton, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { INotification } from "@/core/notifications/lib";
import { NotificationsService } from "@/core/notifications/services";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorPanel } from "@/core/shell/editor/EditorPanel";
import { INotificationEntry, toNotificationEntries } from "@/core/shell/panel/notifications/notification-list";
import { NotificationRow } from "@/core/shell/panel/notifications/NotificationRow";
import { Nullable } from "@/lib/types/general";

/**
 * The notification centre panel.
 *
 * Newest first, flat and unfiltered. The cap is what keeps it readable for now - filters and day
 * separators are worth adding once a session runs long enough to need them, not before.
 */
export function NotificationsPanel(): ReactElement {
  const notificationsService: NotificationsService = useInjection(NotificationsService);
  const settingsService: SettingsService = useInjection(SettingsService);

  const [expandedId, setExpandedId] = useState<Nullable<string>>(null);

  const notifications: Array<INotification> = settingsService.isDevModeEnabled
    ? notificationsService.allNotifications
    : notificationsService.notifications;
  const unreadCount: number = notificationsService.unreadCount;
  const entries: Array<INotificationEntry> = toNotificationEntries(notifications);

  // One record open at a time: two expanded traces leave no room for the outcomes around them.
  const onToggleExpanded = useCallback((id: string) => {
    setExpandedId((it: Nullable<string>) => (it === id ? null : id));
  }, []);

  // Anything visible is read, including what arrives while the panel is open - otherwise the badge
  // counts records the user is looking at, and nothing can dismiss it.
  useEffect(() => {
    notificationsService.markAllRead();
  }, [notificationsService, unreadCount]);

  return (
    <EditorPanel
      className={"h-full"}
      title={"Notifications"}
      actions={
        <Tooltip describeChild title={"Clear all"} placement={"left"}>
          <span>
            <IconButton
              aria-label={"Clear all"}
              disabled={!notifications.length}
              size={"small"}
              onClick={notificationsService.clear}
            >
              <ClearAllIcon fontSize={"small"} />
            </IconButton>
          </span>
        </Tooltip>
      }
    >
      <div className={"h-full overflow-y-auto"}>
        {entries.length ? (
          entries.map((entry: INotificationEntry) => (
            <NotificationRow
              key={entry.notification.id}
              entry={entry}
              isExpanded={entry.notification.id === expandedId}
              onToggleExpanded={onToggleExpanded}
            />
          ))
        ) : (
          <div className={"p-4"}>
            <Typography className={"leading-panel text-text-secondary"} variant={"body2"}>
              Nothing has been reported yet. Command outcomes from every tool collect here.
            </Typography>
          </div>
        )}
      </div>
    </EditorPanel>
  );
}
