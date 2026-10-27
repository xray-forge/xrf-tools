import { BaseComponentProps } from "@xrf/ui/element-types";
import { ArrowOutwardIcon } from "@xrf/ui/icons/arrow-outward";
import { Card, CardActionArea } from "@xrf/ui/material/card";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { CSSProperties, ReactElement } from "react";

import { ToolLauncherGroupLabel } from "@/core/launcher/components/ToolLauncherGroupLabel";
import { ToolLauncherPlannedBadge } from "@/core/launcher/components/ToolLauncherPlannedBadge";
import { toAccentColor, useToolLauncherActions } from "@/core/launcher/lib";
import { EToolStatus, IToolDescriptor, IToolGroup } from "@/core/routing/tool";

interface IToolLauncherCardProps extends BaseComponentProps {
  tool: IToolDescriptor;
  group: IToolGroup;
  /** Names the group on the card itself, for a body with no section heading above it to say so. */
  isGroupNamed?: boolean;
  onOpen: (tool: IToolDescriptor) => void;
}

/**
 * One tool on the root catalog grid.
 */
export function ToolLauncherCard({
  "data-testid": dataTestId = "tool-launcher-card",
  id,
  className,
  tool,
  group,
  isGroupNamed,
  onOpen,
}: IToolLauncherCardProps): ReactElement {
  const { onWarm, onClick } = useToolLauncherActions(tool, onOpen);

  const isPlanned: boolean = tool.status === EToolStatus.PLANNED;

  return (
    <Card data-testid={dataTestId} id={id} className={cn("h-full", className)}>
      <CardActionArea
        aria-label={tool.label}
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
              {tool.icon}
            </span>

            {isPlanned ? (
              <ToolLauncherPlannedBadge />
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
              {tool.label}
            </Typography>

            <Typography className={"line-clamp-2 min-h-[2lh] leading-[1.45] text-text-secondary"} variant={"body2"}>
              {tool.description}
            </Typography>
          </div>

          {isGroupNamed ? <ToolLauncherGroupLabel className={"mt-auto"} group={group} /> : null}
        </div>
      </CardActionArea>
    </Card>
  );
}
