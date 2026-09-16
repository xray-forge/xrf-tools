import { Card, CardActionArea, cardActionAreaClasses, SxProps, Theme, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ApplicationLauncherGroupLabel } from "@/core/launcher/components/ApplicationLauncherGroupLabel";
import { ApplicationLauncherPlannedBadge } from "@/core/launcher/components/ApplicationLauncherPlannedBadge";
import { toAccentColor, useApplicationLauncherActions } from "@/core/launcher/lib";
import { EApplicationStatus, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** The focus ring, which only MUI's own class can carry. */
const ACTION_AREA_SX: SxProps<Theme> = {
  [`&.${cardActionAreaClasses.focusVisible}`]: {
    outline: "2px solid",
    outlineColor: "primary.main",
    outlineOffset: -2,
  },
};

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

  return (
    <Card data-testid={dataTestId} id={id} className={cn("h-full", className)}>
      <CardActionArea
        aria-label={application.label}
        className={"block h-full"}
        sx={ACTION_AREA_SX}
        onFocus={onWarm}
        onMouseEnter={onWarm}
        onClick={onClick}
      >
        <div className={"flex h-full flex-col gap-1 p-2.5"}>
          <div className={"flex min-w-0 items-center gap-1.5"}>
            <span
              aria-hidden={true}
              className={"flex shrink-0 [&>svg]:text-[1.125rem]"}
              style={{ color: toAccentColor(group.accent) }}
            >
              {application.icon}
            </span>

            <Typography variant={"subtitle2"} className={"line-clamp-2 min-w-0 grow leading-[1.3] text-text-primary"}>
              {application.label}
            </Typography>

            {application.status === EApplicationStatus.PLANNED ? <ApplicationLauncherPlannedBadge /> : null}
          </div>

          <Typography variant={"body2"} className={"line-clamp-2 leading-[1.35] text-text-secondary"}>
            {application.description}
          </Typography>

          {isGroupNamed ? <ApplicationLauncherGroupLabel group={group} /> : null}
        </div>
      </CardActionArea>
    </Card>
  );
}
