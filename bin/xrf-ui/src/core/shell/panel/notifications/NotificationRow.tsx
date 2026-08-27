import { default as CheckCircleOutlineIcon } from "@mui/icons-material/CheckCircleOutlineOutlined";
import { default as ContentCopyIcon } from "@mui/icons-material/ContentCopy";
import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { default as ExpandLessIcon } from "@mui/icons-material/ExpandLess";
import { default as ExpandMoreIcon } from "@mui/icons-material/ExpandMore";
import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { default as TerminalIcon } from "@mui/icons-material/Terminal";
import { default as WarningAmberIcon } from "@mui/icons-material/WarningAmber";
import { Box, Chip, Collapse, IconButton, Tooltip, Typography } from "@mui/material";
import { format } from "date-fns";
import { ReactElement, ReactNode, useCallback, useState } from "react";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { ENotificationSeverity, INotification } from "@/core/notifications/lib";
import { IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

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
  notification: INotification;
}

/**
 * One recorded outcome.
 *
 * `details` is collapsed by default: a stack or a path list is what makes a record useful once you are
 * already reading it, and what makes the log unscannable before that.
 */
export function NotificationRow({ notification }: INotificationRowProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [isExpanded, setExpanded] = useState<boolean>(false);

  const application: Nullable<IApplicationDescriptor> = APPLICATION_CATALOG.findApplicationById(notification.source);
  const group: Nullable<IApplicationGroup> = application
    ? null
    : APPLICATION_CATALOG.findApplicationGroupById(notification.source);

  const createdAt: Date = new Date(notification.createdAt);
  const isDev: boolean = notification.severity === ENotificationSeverity.DEV;

  const onCopyDetails = useCallback(() => {
    navigator.clipboard?.writeText(notification.details ?? "").catch((error: unknown) => {
      log.error("Failed to copy notification details:", error);
    });
  }, [log, notification.details]);

  return (
    <Box
      sx={{
        paddingX: 1.5,
        paddingY: 1,
        borderBottom: 1,
        borderColor: "divider",
        // A dev trace is subordinate to everything around it: struck out of the normal reading order by
        // the dashed edge, and dimmed so a panel full of them still scans for real outcomes.
        ...(isDev
          ? {
              borderLeft: "2px dashed",
              borderLeftColor: "divider",
              backgroundColor: "action.hover",
              opacity: 0.85,
            }
          : {}),
      }}
    >
      <Box sx={{ display: "flex", alignItems: "flex-start", gap: 1 }}>
        <Box sx={{ display: "flex", paddingTop: 0.25, color: SEVERITY_COLORS[notification.severity] }}>
          {SEVERITY_ICONS[notification.severity]}
        </Box>

        <Box sx={{ flexGrow: 1, minWidth: 0 }}>
          <Typography
            variant={"body2"}
            sx={{
              overflowWrap: "anywhere",
              ...(isDev ? { fontFamily: "monospace", fontSize: "0.78rem", color: "text.secondary" } : {}),
            }}
          >
            {notification.title}
          </Typography>

          <Box sx={{ display: "flex", alignItems: "center", gap: 0.75 }}>
            {isDev ? (
              <Chip
                label={"DEV"}
                size={"small"}
                variant={"outlined"}
                sx={{ height: 16, fontSize: "0.6rem", letterSpacing: 0.5, "& .MuiChip-label": { paddingX: 0.5 } }}
              />
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
          <IconButton
            aria-label={isExpanded ? "Hide details" : "Show details"}
            aria-pressed={isExpanded}
            size={"small"}
            onClick={() => setExpanded((it: boolean) => !it)}
          >
            {isExpanded ? <ExpandLessIcon fontSize={"small"} /> : <ExpandMoreIcon fontSize={"small"} />}
          </IconButton>
        ) : null}
      </Box>

      {notification.details ? (
        <Collapse in={isExpanded} unmountOnExit>
          <Box sx={{ display: "flex", alignItems: "flex-start", gap: 0.5, marginTop: 0.5 }}>
            <Typography
              variant={"caption"}
              sx={{
                flexGrow: 1,
                minWidth: 0,
                padding: 1,
                borderRadius: 1,
                fontFamily: "monospace",
                whiteSpace: "pre-wrap",
                overflowWrap: "anywhere",
                backgroundColor: "background.paper",
              }}
            >
              {notification.details}
            </Typography>

            <Tooltip describeChild title={"Copy details"} placement={"left"}>
              <IconButton aria-label={"Copy details"} size={"small"} onClick={onCopyDetails}>
                <ContentCopyIcon fontSize={"small"} />
              </IconButton>
            </Tooltip>
          </Box>
        </Collapse>
      ) : null}
    </Box>
  );
}
