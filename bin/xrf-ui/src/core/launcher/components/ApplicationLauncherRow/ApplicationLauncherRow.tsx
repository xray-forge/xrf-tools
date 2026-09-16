import { ListItem, ListItemButton, SxProps, Theme, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ApplicationLauncherGroupLabel } from "@/core/launcher/components/ApplicationLauncherGroupLabel";
import { ApplicationLauncherPlannedBadge } from "@/core/launcher/components/ApplicationLauncherPlannedBadge";
import { toAccentColor, useApplicationLauncherActions } from "@/core/launcher/lib";
import { EApplicationStatus, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { TREE } from "@/core/theme/tokens";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

const ROW_COLUMNS: string = `${TREE.iconWidth}px 240px minmax(0, 1fr)`;
const ROW_SX: SxProps<Theme> = { gridTemplateColumns: ROW_COLUMNS };
const NAMED_ROW_SX: SxProps<Theme> = { gridTemplateColumns: `${ROW_COLUMNS} 132px` };

interface IApplicationLauncherRowProps extends BaseComponentProps {
  application: IApplicationDescriptor;
  group: IApplicationGroup;
  /** Names the group on the row itself, for a list with no separator above it to say so. */
  isGroupNamed?: boolean;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * One application on the root catalog list.
 */
export function ApplicationLauncherRow({
  "data-testid": dataTestId = "application-launcher-row",
  id,
  className,
  application,
  group,
  isGroupNamed,
  onOpen,
}: IApplicationLauncherRowProps): ReactElement {
  const { onWarm, onClick } = useApplicationLauncherActions(application, onOpen);

  return (
    <ListItem data-testid={dataTestId} id={id} className={cn("block", className)} disablePadding={true}>
      <ListItemButton
        aria-label={application.label}
        className={"grid h-tree-row items-center gap-2 px-2 py-0"}
        sx={isGroupNamed ? NAMED_ROW_SX : ROW_SX}
        onFocus={onWarm}
        onMouseEnter={onWarm}
        onClick={onClick}
      >
        <span
          aria-hidden={true}
          className={"flex shrink-0 [&>svg]:text-tree-icon"}
          style={{ color: toAccentColor(group.accent) }}
        >
          {application.icon}
        </span>

        <div className={"flex min-w-0 items-center gap-1.5"}>
          <Typography
            className={"min-w-0 overflow-hidden font-medium text-ellipsis whitespace-nowrap text-text-primary"}
            variant={"body2"}
          >
            {application.label}
          </Typography>

          {application.status === EApplicationStatus.PLANNED ? <ApplicationLauncherPlannedBadge /> : null}
        </div>

        <Typography
          variant={"body2"}
          className={"min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-text-secondary"}
        >
          {application.description}
        </Typography>

        {isGroupNamed ? <ApplicationLauncherGroupLabel group={group} /> : null}
      </ListItemButton>
    </ListItem>
  );
}
