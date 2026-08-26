import { Typography } from "@mui/material";
import { ReactElement } from "react";

/**
 * Marks a tool the roster names but does not implement yet.
 */
export function ApplicationLauncherPlannedBadge(): ReactElement {
  return (
    <Typography
      component={"span"}
      variant={"caption"}
      sx={{
        flexShrink: 0,
        alignSelf: "flex-start",
        paddingX: 0.75,
        color: "text.secondary",
        border: "1px solid",
        borderColor: "divider",
        borderRadius: 1,
        fontSize: "0.625rem",
        fontWeight: 600,
        lineHeight: "17px",
        whiteSpace: "nowrap",
      }}
    >
      Planned
    </Typography>
  );
}
