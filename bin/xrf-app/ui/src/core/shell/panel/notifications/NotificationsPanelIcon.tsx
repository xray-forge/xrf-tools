import { default as NotificationsIcon } from "@mui/icons-material/Notifications";
import { Badge, badgeClasses } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ENotificationSeverity } from "@/core/notifications/lib";
import { NotificationsService } from "@/core/notifications/services";
import { LAYOUT } from "@/core/theme/tokens";

const BADGE_COLORS: Record<ENotificationSeverity, "default" | "success" | "info" | "warning" | "error"> = {
  [ENotificationSeverity.DEV]: "default",
  [ENotificationSeverity.ERROR]: "error",
  [ENotificationSeverity.INFO]: "info",
  [ENotificationSeverity.SUCCESS]: "success",
  [ENotificationSeverity.WARNING]: "warning",
};

/**
 * The stripe icon, badged with what has not been read yet.
 */
export function NotificationsPanelIcon(): ReactElement {
  const notificationsService: NotificationsService = useInjection(NotificationsService);

  const severity: Nullable<ENotificationSeverity> = notificationsService.highestUnreadSeverity;

  return (
    <Badge
      className={"size-rail-button items-center justify-center"}
      badgeContent={notificationsService.unreadCount}
      color={severity ? BADGE_COLORS[severity] : "default"}
      max={99}
      overlap={"circular"}
      sx={{
        [`& .${badgeClasses.badge}`]: {
          top: 0,
          right: 0,
          transform: "scale(1)",
          transformOrigin: "top right",
          height: LAYOUT.railBadgeHeight,
          minWidth: LAYOUT.railBadgeMinWidth,
          maxWidth: LAYOUT.railButtonSize,
          padding: `0 ${LAYOUT.railBadgePaddingX}px`,
          fontSize: LAYOUT.railBadgeFontSize,
          [`&.${badgeClasses.invisible}`]: { transform: "scale(0)" },
        },
      }}
    >
      <NotificationsIcon />
    </Badge>
  );
}
