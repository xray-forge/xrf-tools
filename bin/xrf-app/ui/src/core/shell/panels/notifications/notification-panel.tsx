import { NotificationsPanel } from "@/core/shell/panels/notifications/NotificationsPanel";
import { NotificationsPanelIcon } from "@/core/shell/panels/notifications/NotificationsPanelIcon";
import { IShellPanel } from "@/core/shell/panels/shell-panel";

/** The notification log owned by the frame and available in every tool. */
export const NOTIFICATIONS_PANEL: IShellPanel = {
  icon: <NotificationsPanelIcon />,
  id: "notifications",
  isOpenByDefault: false,
  label: "Notifications",
  render: () => <NotificationsPanel />,
  side: "right",
};
