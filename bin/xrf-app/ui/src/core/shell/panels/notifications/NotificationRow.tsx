import { formatCount } from "@xrf/format";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { CheckCircleOutlineOutlinedIcon as CheckCircleOutlineIcon } from "@xrf/ui/icons/check-circle-outline-outlined";
import { ContentCopyIcon } from "@xrf/ui/icons/content-copy";
import { ErrorOutlineOutlinedIcon as ErrorOutlineIcon } from "@xrf/ui/icons/error-outline-outlined";
import { ExpandLessIcon } from "@xrf/ui/icons/expand-less";
import { ExpandMoreIcon } from "@xrf/ui/icons/expand-more";
import { InfoOutlinedIcon } from "@xrf/ui/icons/info-outlined";
import { TerminalIcon } from "@xrf/ui/icons/terminal";
import { WarningAmberIcon } from "@xrf/ui/icons/warning-amber";
import { Box } from "@xrf/ui/material/box";
import { Button } from "@xrf/ui/material/button";
import { IconButton } from "@xrf/ui/material/icon-button";
import { Tooltip } from "@xrf/ui/material/tooltip";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { format } from "date-fns";
import { ReactElement, ReactNode, useCallback, useState } from "react";

import { ENotificationSeverity } from "@/core/notifications/lib";
import { IToolDescriptor, IToolGroup } from "@/core/routing/tool";
import { INotificationEntry, isAttentionSeverity } from "@/core/shell/panels/notifications/notification-list";
import { ICopiedText, useCopiedText } from "@/lib/react/use-copied-text";
import { TOOL_CATALOG } from "@/ToolCatalog";

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

interface INotificationRowProps {
  entry: INotificationEntry;
  isExpanded: boolean;
  onToggleExpanded: (id: string) => void;
}

/**
 * One recorded outcome: the title, then the tool and the time it happened.
 */
export function NotificationRow({ entry, isExpanded, onToggleExpanded }: INotificationRowProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [isFullShown, setFullShown] = useState<boolean>(false);

  const { notification, repeatCount } = entry;

  const tool: Nullable<IToolDescriptor> = TOOL_CATALOG.findToolById(notification.source);
  const group: Nullable<IToolGroup> = tool ? null : TOOL_CATALOG.findToolGroupById(notification.source);

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

  const onCopyFailed = useCallback((error: unknown) => log.error("Failed to copy notification details:", error), [log]);

  const { copy }: ICopiedText = useCopiedText(onCopyFailed);

  const onCopyDetails = useCallback(() => copy(notification.details ?? ""), [copy, notification.details]);

  return (
    <div className={"group border-b border-divider px-3 py-1.5"}>
      <div className={"flex items-start gap-2"}>
        <Box className={"flex shrink-0 pt-0.5"} sx={{ color: SEVERITY_COLORS[notification.severity] }}>
          {SEVERITY_ICONS[notification.severity]}
        </Box>

        <div className={"min-w-0 grow"}>
          <Typography
            className={cn("wrap-anywhere", isDev && "monospace text-text-secondary")}
            variant={"body2"}
            sx={isEmphasized ? { color: SEVERITY_COLORS[notification.severity] } : undefined}
          >
            {notification.title}
          </Typography>

          <div className={"flex flex-wrap items-center gap-1.5"}>
            {repeatCount > 1 ? (
              <Typography
                className={"rounded-control border border-divider px-1 text-text-secondary"}
                variant={"caption"}
              >
                ×{formatCount(repeatCount)}
              </Typography>
            ) : null}

            <Typography className={"text-text-secondary"} variant={"caption"}>
              {tool?.label ?? group?.label ?? notification.source}
            </Typography>

            <Tooltip describeChild title={format(createdAt, "yyyy-MM-dd HH:mm:ss")} placement={"left"}>
              <Typography className={"text-text-secondary opacity-70"} variant={"caption"}>
                {format(createdAt, "HH:mm:ss")}
              </Typography>
            </Tooltip>
          </div>
        </div>

        {notification.details ? (
          <div className={"flex shrink-0"}>
            <div className={"flex opacity-0 group-focus-within:opacity-100 group-hover:opacity-100"}>
              <Tooltip describeChild title={"Copy details"} placement={"left"}>
                <IconButton aria-label={"Copy details"} className={"p-0.5"} size={"small"} onClick={onCopyDetails}>
                  <ContentCopyIcon className={"text-panel-action-icon"} />
                </IconButton>
              </Tooltip>
            </div>

            <IconButton
              aria-label={isExpanded ? "Hide details" : "Show details"}
              aria-pressed={isExpanded}
              className={"p-0.5"}
              size={"small"}
              onClick={() => onToggleExpanded(notification.id)}
            >
              {isExpanded ? (
                <ExpandLessIcon className={"text-panel-action-icon"} />
              ) : (
                <ExpandMoreIcon className={"text-panel-action-icon"} />
              )}
            </IconButton>
          </div>
        ) : null}
      </div>

      {isBodyShown ? (
        <div className={"mt-1 rounded-surface border border-divider bg-well p-2"}>
          <Typography
            className={cn(
              "monospace m-0 leading-normal text-text-secondary",
              isExpanded ? "wrap-anywhere whitespace-pre-wrap" : "overflow-hidden text-ellipsis whitespace-nowrap"
            )}
            component={"pre"}
          >
            {body}
          </Typography>

          {isExpanded && isClamped ? (
            <Button className={"mt-1 min-w-0 px-1"} size={"small"} onClick={() => setFullShown(true)}>
              Show all {formatCount(lines.length)} lines
            </Button>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
