import { BaseComponentProps } from "@xrf/ui/element-types";
import { ListItem, ListItemButton } from "@xrf/ui/material/list";
import { Typography } from "@xrf/ui/material/typography";
import { cn, TREE } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { toAccentColor, useToolLauncherActions } from "@/core/launcher/lib";
import { EToolStatus, IToolDescriptor, IToolGroup } from "@/core/routing/tool";

import { ToolLauncherGroupLabel } from "../ToolLauncherGroupLabel";
import { ToolLauncherPlannedBadge } from "../ToolLauncherPlannedBadge";

const ROW_COLUMNS: string = `${TREE.iconWidth}px 240px minmax(0, 1fr)`;
const NAMED_ROW_COLUMNS: string = `${ROW_COLUMNS} 132px`;

interface IToolLauncherRowProps extends BaseComponentProps {
  tool: IToolDescriptor;
  group: IToolGroup;
  /** Names the group on the row itself, for a list with no separator above it to say so. */
  isGroupNamed?: boolean;
  onOpen: (tool: IToolDescriptor) => void;
}

/**
 * One tool on the root catalog list.
 */
export function ToolLauncherRow({
  "data-testid": dataTestId = "tool-launcher-row",
  id,
  className,
  tool,
  group,
  isGroupNamed,
  onOpen,
}: IToolLauncherRowProps): ReactElement {
  const { onWarm, onClick } = useToolLauncherActions(tool, onOpen);

  return (
    <ListItem data-testid={dataTestId} id={id} className={cn("block", className)} disablePadding={true}>
      <ListItemButton
        aria-label={tool.label}
        className={"grid h-tree-row items-center gap-2 px-2 py-0"}
        style={{ gridTemplateColumns: isGroupNamed ? NAMED_ROW_COLUMNS : ROW_COLUMNS }}
        onFocus={onWarm}
        onMouseEnter={onWarm}
        onClick={onClick}
      >
        <span
          aria-hidden={true}
          className={"flex shrink-0 [&>svg]:text-tree-icon"}
          style={{ color: toAccentColor(group.accent) }}
        >
          {tool.icon}
        </span>

        <div className={"flex min-w-0 items-center gap-1.5"}>
          <Typography
            className={"min-w-0 overflow-hidden font-medium text-ellipsis whitespace-nowrap text-text-primary"}
            variant={"body2"}
          >
            {tool.label}
          </Typography>

          {tool.status === EToolStatus.PLANNED ? <ToolLauncherPlannedBadge /> : null}
        </div>

        <Typography
          variant={"body2"}
          className={"min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-text-secondary"}
        >
          {tool.description}
        </Typography>

        {isGroupNamed ? <ToolLauncherGroupLabel group={group} /> : null}
      </ListItemButton>
    </ListItem>
  );
}
