import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { toAccentColor } from "@/core/launcher/lib";
import { IToolGroup } from "@/core/routing/tool";

interface IToolLauncherGroupLabelProps extends BaseComponentProps {
  group: IToolGroup;
}

/**
 * The group a tool belongs to, wherever no section heading is there to say it.
 */
export function ToolLauncherGroupLabel({
  "data-testid": dataTestId = "tool-launcher-group-label",
  id,
  className,
  group,
}: IToolLauncherGroupLabelProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-w-0 items-center gap-1.5", className)}>
      <span
        aria-hidden={true}
        className={"size-1.5 shrink-0 rounded-full"}
        style={{ backgroundColor: toAccentColor(group.accent) }}
      />

      <Typography
        className={"min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-text-secondary"}
        variant={"caption"}
      >
        {group.label}
      </Typography>
    </div>
  );
}
