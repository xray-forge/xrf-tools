import { Box, Theme, Typography } from "@mui/material";
import { ReactElement } from "react";

import { IApplicationGroup } from "@/core/routing/application";

export interface IApplicationLauncherGroupLabelProps {
  group: IApplicationGroup;
}

/**
 * The group a tool belongs to, wherever no section heading is there to say it.
 */
export function ApplicationLauncherGroupLabel({ group }: IApplicationLauncherGroupLabelProps): ReactElement {
  return (
    <Box sx={{ display: "flex", alignItems: "center", gap: 0.75, minWidth: 0 }}>
      <Box
        aria-hidden={true}
        sx={(theme: Theme) => ({
          flexShrink: 0,
          width: 6,
          height: 6,
          borderRadius: "50%",
          backgroundColor: group.accent.light,
          ...theme.applyStyles("dark", { backgroundColor: group.accent.dark }),
        })}
      />

      <Typography
        variant={"caption"}
        sx={{
          minWidth: 0,
          overflow: "hidden",
          color: "text.secondary",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
      >
        {group.label}
      </Typography>
    </Box>
  );
}
