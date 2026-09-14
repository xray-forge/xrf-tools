import { Box, svgIconClasses, Theme, Typography } from "@mui/material";
import { ReactElement } from "react";

import { IApplicationGroup } from "@/core/routing/application";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IApplicationLauncherGroupHeadingProps extends BaseComponentProps {
  group: IApplicationGroup;
  count: number;
}

/**
 * The heading that opens one group's tools, in either view.
 */
export function ApplicationLauncherGroupHeading({
  "data-testid": dataTestId = "application-launcher-group-heading",
  id,
  className,
  group,
  count,
}: IApplicationLauncherGroupHeadingProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", alignItems: "center", gap: 0.75, minWidth: 0 }}
    >
      <Box
        aria-hidden={true}
        sx={(theme: Theme) => ({
          display: "flex",
          color: group.accent.light,
          [`& .${svgIconClasses.root}`]: { fontSize: 16 },
          ...theme.applyStyles("dark", { color: group.accent.dark }),
        })}
      >
        {group.icon}
      </Box>

      <Typography component={"h2"} variant={"subtitle2"} sx={{ color: "text.primary", fontWeight: 600 }}>
        {group.label}
      </Typography>

      <Typography variant={"caption"} sx={{ color: "text.secondary", opacity: 0.7 }}>
        {count}
      </Typography>
    </Box>
  );
}
