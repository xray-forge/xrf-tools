import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { toAccentColor } from "@/core/launcher/lib";
import { IToolGroup } from "@/core/routing/tool";

interface IToolLauncherGroupHeadingProps extends BaseComponentProps {
  group: IToolGroup;
}

/**
 * The heading that opens one group's tools, in either view.
 */
export function ToolLauncherGroupHeading({
  "data-testid": dataTestId = "tool-launcher-group-heading",
  id,
  className,
  group,
}: IToolLauncherGroupHeadingProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-w-0 items-center gap-1.5", className)}>
      <span aria-hidden={true} className={"flex [&>svg]:text-base"} style={{ color: toAccentColor(group.accent) }}>
        {group.icon}
      </span>

      <Typography component={"h2"} variant={"subtitle2"} className={"font-semibold text-text-primary"}>
        {group.label}
      </Typography>
    </div>
  );
}
