import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { ClearAllIcon } from "@xrf/ui/icons/clear-all";
import { Typography } from "@xrf/ui/typography";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { INotification } from "@/core/notifications/lib";
import { NotificationsService } from "@/core/notifications/services";
import { SettingsService } from "@/core/settings/services/settings.service";
import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import { INotificationEntry, toNotificationEntries } from "@/core/shell/panels/notifications/notification-list";
import { NotificationRow } from "@/core/shell/panels/notifications/NotificationRow";

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
        <EditorIconAction
          label={"Clear all"}
          description={notifications.length ? "Clear every notification" : "Nothing to clear"}
          icon={<ClearAllIcon />}
          isDisabled={!notifications.length}
          onClick={notificationsService.clear}
        />
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
