import { default as CheckCircleOutlineIcon } from "@mui/icons-material/CheckCircleOutlineOutlined";
import { default as ContentCopyIcon } from "@mui/icons-material/ContentCopy";
import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { default as ExpandLessIcon } from "@mui/icons-material/ExpandLess";
import { default as ExpandMoreIcon } from "@mui/icons-material/ExpandMore";
import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { default as TerminalIcon } from "@mui/icons-material/Terminal";
import { default as WarningAmberIcon } from "@mui/icons-material/WarningAmber";
import { Box, Button, IconButton, Tooltip, Typography } from "@mui/material";
import { format } from "date-fns";
import { ReactElement, ReactNode, useCallback, useState } from "react";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { INotificationEntry, isAttentionSeverity } from "@/core/shell/panel/notifications/notification-list";
import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Lines of `details` shown inline before the rest has to be asked for. */
const DETAILS_LINE_LIMIT: number = 8;

const SEVERITY_ICONS: Record<ENotificationSeverity, ReactNode> = {
  [ENotificationSeverity.DEV]: <TerminalIcon fontSize={"small"} />,
  [ENotificationSeverity.ERROR]: <ErrorOutlineIcon fontSize={"small"} />,
  [ENotificationSeverity.INFO]: <InfoOutlinedIcon fontSize={"small"} />,
  [ENotificationSeverity.SUCCESS]: <CheckCircleOutlineIcon fontSize={"small"} />,
  [ENotificationSeverity.WARNING]: <WarningAmberIcon fontSize={"small"} />,
};

const SEVERITY_COLORS: Record<ENotificationSeverity, string> = {
  [ENotificationSeverity.DEV]: "text.disabled",
  [ENotificationSeverity.ERROR]: "error.main",
  [ENotificationSeverity.INFO]: "info.main",
  [ENotificationSeverity.SUCCESS]: "success.main",
  [ENotificationSeverity.WARNING]: "warning.main",
};

export interface INotificationRowProps {
  entry: INotificationEntry;
  isExpanded: boolean;
  onToggleExpanded: (id: string) => void;
}

/**
 * One recorded outcome: the title, then the tool and the time it happened.
 *
 * The title wraps and the second line always states its tool. Neither is allowed to truncate - a title
 * cut to an ellipsis and a tool name that shrinks away are the two things a log cannot afford to lose,
 * whatever it buys in rows per screen.
 *
 * Severity is carried by the icon, and by the title colour for the two severities that have to compete
 * with a pile of routine successes. Nothing tints a row: a filled background costs more attention than
 * it buys, and a panel of dev traces would be one continuous block of it.
 *
 * `details` stays collapsed - a stack or a path list is what makes a record useful once you are already
 * reading it, and what makes the log unscannable before that - except for the first line of a failure,
 * which is usually the reason it failed.
 */
export function NotificationRow({ entry, isExpanded, onToggleExpanded }: INotificationRowProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [isFullShown, setFullShown] = useState<boolean>(false);

  const { notification, repeatCount } = entry;

  const application: Nullable<IApplicationDescriptor> = APPLICATION_CATALOG.findApplicationById(notification.source);
  const group: Nullable<IApplicationGroup> = application
    ? null
    : APPLICATION_CATALOG.findApplicationGroupById(notification.source);

  const createdAt: Date = new Date(notification.createdAt);
  const isDev: boolean = notification.severity === ENotificationSeverity.DEV;
  const isEmphasized: boolean = isAttentionSeverity(notification.severity);
  const lines: Array<string> = notification.details ? notification.details.split("\n") : [];
  const isClamped: boolean = !isFullShown && lines.length > DETAILS_LINE_LIMIT;
  // The same body in two states: one line of it while collapsed, all of it once opened. Either state
  // ends in an ellipsis when it is holding something back, so nothing ever reads as the whole of it.
  const isBodyShown: boolean = Boolean(notification.details) && (isExpanded || isEmphasized);
  const expandedBody: string = isClamped
    ? `${lines.slice(0, DETAILS_LINE_LIMIT).join("\n")}\n…`
    : (notification.details ?? "");
  const collapsedBody: string = `${lines[0] ?? ""}${lines.length > 1 ? " …" : ""}`;
  const body: string = isExpanded ? expandedBody : collapsedBody;

  const onCopyDetails = useCallback(() => {
    navigator.clipboard?.writeText(notification.details ?? "").catch((error: unknown) => {
      log.error("Failed to copy notification details:", error);
    });
  }, [log, notification.details]);

  return (
    <Box
      sx={{
        paddingX: 1.5,
        paddingY: 0.75,
        borderBottom: 1,
        borderColor: "divider",
        // Revealed rather than laid out: a control parked beside the details holds its width for the
        // whole height of the block, and `focus-within` keeps it reachable without a pointer.
        "&:hover .notification-row-actions, &:focus-within .notification-row-actions": { opacity: 1 },
      }}
    >
      <Box sx={{ display: "flex", alignItems: "flex-start", gap: 1 }}>
        <Box sx={{ display: "flex", flexShrink: 0, paddingTop: 0.25, color: SEVERITY_COLORS[notification.severity] }}>
          {SEVERITY_ICONS[notification.severity]}
        </Box>

        <Box sx={{ flexGrow: 1, minWidth: 0 }}>
          <Typography
            variant={"body2"}
            sx={{
              overflowWrap: "anywhere",
              ...(isEmphasized ? { color: SEVERITY_COLORS[notification.severity] } : {}),
              ...(isDev ? { ...MONOSPACE, color: "text.secondary" } : {}),
            }}
          >
            {notification.title}
          </Typography>

          <Box sx={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 0.75 }}>
            {repeatCount > 1 ? (
              <Typography
                variant={"caption"}
                sx={{ paddingX: 0.5, border: 1, borderColor: "divider", borderRadius: 0.5, color: "text.secondary" }}
              >
                ×{repeatCount}
              </Typography>
            ) : null}

            <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
              {application?.label ?? group?.label ?? notification.source}
            </Typography>

            <Tooltip describeChild title={format(createdAt, "yyyy-MM-dd HH:mm:ss")} placement={"left"}>
              <Typography variant={"caption"} sx={{ color: "text.secondary", opacity: 0.7 }}>
                {format(createdAt, "HH:mm:ss")}
              </Typography>
            </Tooltip>
          </Box>
        </Box>

        {notification.details ? (
          <Box sx={{ display: "flex", flexShrink: 0 }}>
            <Box className={"notification-row-actions"} sx={{ display: "flex", opacity: 0 }}>
              <Tooltip describeChild title={"Copy details"} placement={"left"}>
                <IconButton aria-label={"Copy details"} size={"small"} sx={{ padding: 0.25 }} onClick={onCopyDetails}>
                  <ContentCopyIcon sx={{ fontSize: PANEL.actionIconSize }} />
                </IconButton>
              </Tooltip>
            </Box>

            <IconButton
              aria-label={isExpanded ? "Hide details" : "Show details"}
              aria-pressed={isExpanded}
              size={"small"}
              sx={{ padding: 0.25 }}
              onClick={() => onToggleExpanded(notification.id)}
            >
              {isExpanded ? (
                <ExpandLessIcon sx={{ fontSize: PANEL.actionIconSize }} />
              ) : (
                <ExpandMoreIcon sx={{ fontSize: PANEL.actionIconSize }} />
              )}
            </IconButton>
          </Box>
        ) : null}
      </Box>

      {isBodyShown ? (
        <Box
          sx={{
            marginTop: 0.5,
            padding: 1,
            borderRadius: 1,
            backgroundColor: "background.paper",
          }}
        >
          <Typography
            component={"pre"}
            sx={{
              margin: 0,
              ...MONOSPACE,
              lineHeight: 1.45,
              color: "text.secondary",
              ...(isExpanded
                ? { whiteSpace: "pre-wrap", overflowWrap: "anywhere" }
                : { whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }),
            }}
          >
            {body}
          </Typography>

          {isExpanded && isClamped ? (
            <Button
              size={"small"}
              sx={{ minWidth: 0, marginTop: 0.5, paddingX: 0.5 }}
              onClick={() => setFullShown(true)}
            >
              Show all {lines.length} lines
            </Button>
          ) : null}
        </Box>
      ) : null}
    </Box>
  );
}
