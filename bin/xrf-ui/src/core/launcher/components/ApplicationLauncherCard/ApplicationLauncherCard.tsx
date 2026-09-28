import { default as ArrowOutwardIcon } from "@mui/icons-material/ArrowOutward";
import { Card, CardActionArea, Typography } from "@mui/material";
import { CSSProperties, ReactElement } from "react";

import { ApplicationLauncherGroupLabel } from "@/core/launcher/components/ApplicationLauncherGroupLabel";
import { ApplicationLauncherPlannedBadge } from "@/core/launcher/components/ApplicationLauncherPlannedBadge";
import { toAccentColor, useApplicationLauncherActions } from "@/core/launcher/lib";
import { EApplicationStatus, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IApplicationLauncherCardProps extends BaseComponentProps {
  application: IApplicationDescriptor;
  group: IApplicationGroup;
  /** Names the group on the card itself, for a body with no section heading above it to say so. */
  isGroupNamed?: boolean;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * One application on the root catalog grid.
 */
export function ApplicationLauncherCard({
  "data-testid": dataTestId = "application-launcher-card",
  id,
  className,
  application,
  group,
  isGroupNamed,
  onOpen,
}: IApplicationLauncherCardProps): ReactElement {
  const { onWarm, onClick } = useApplicationLauncherActions(application, onOpen);

  const isPlanned: boolean = application.status === EApplicationStatus.PLANNED;

  return (
    <Card data-testid={dataTestId} id={id} className={cn("h-full", className)}>
      <CardActionArea
        aria-label={application.label}
        className={cn(
          "group block h-full",
          "focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-primary"
        )}
        onFocus={onWarm}
        onMouseEnter={onWarm}
        onClick={onClick}
      >
        <div className={"flex h-full flex-col gap-3 p-4"}>
          <div className={"flex items-start justify-between gap-2"}>
            <span
              aria-hidden={true}
              className={cn(
                "grid size-12 shrink-0 place-items-center rounded-[12px] text-(--launcher-accent) [&>svg]:text-[1.75rem]",
                // The edge carries the group's colour, so the fill can stay faint enough not to compete with the label.
                "bg-(--launcher-accent)/5 inset-ring inset-ring-(--launcher-accent)/28",
                "transition-[background-color,box-shadow] duration-150",
                "group-hover:bg-(--launcher-accent)/9 group-hover:inset-ring-(--launcher-accent)/50",
                "group-focus-visible:bg-(--launcher-accent)/9 group-focus-visible:inset-ring-(--launcher-accent)/50",
                isPlanned && "opacity-60"
              )}
              style={{ "--launcher-accent": toAccentColor(group.accent) } as CSSProperties}
            >
              {application.icon}
            </span>

            {isPlanned ? (
              <ApplicationLauncherPlannedBadge />
            ) : (
              <ArrowOutwardIcon
                aria-hidden={true}
                className={cn(
                  "text-editor-action-icon text-primary opacity-0 transition-opacity duration-150",
                  "group-hover:opacity-100 group-focus-visible:opacity-100"
                )}
              />
            )}
          </div>

          <div className={"flex min-w-0 flex-col gap-1"}>
            <Typography
              className={cn("line-clamp-2 leading-[1.3] font-semibold text-text-primary", isPlanned && "opacity-60")}
              component={"h3"}
              variant={"subtitle1"}
            >
              {application.label}
            </Typography>

            <Typography className={"line-clamp-2 min-h-[2lh] leading-[1.45] text-text-secondary"} variant={"body2"}>
              {application.description}
            </Typography>
          </div>

          {isGroupNamed ? <ApplicationLauncherGroupLabel className={"mt-auto"} group={group} /> : null}
        </div>
      </CardActionArea>
    </Card>
  );
}
