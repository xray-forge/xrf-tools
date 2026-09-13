import { Box, Typography } from "@mui/material";
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
      <Box aria-hidden={true} sx={{ display: "flex", color: "text.secondary", "& .MuiSvgIcon-root": { fontSize: 16 } }}>
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
