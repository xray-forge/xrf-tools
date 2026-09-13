import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ISettingsStatProps extends BaseComponentProps {
  label: string;
  value: string;
}

/**
 * One figure of a summary, named.
 */
export function SettingsStat({
  "data-testid": dataTestId = "settings-stat",
  className,
  id,
  label,
  value,
}: ISettingsStatProps): ReactElement {
  return (
    <Box data-testid={dataTestId} className={className} id={id} sx={{ minWidth: 104 }}>
      <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block" }}>
        {label}
      </Typography>

      <Typography variant={"body2"}>{value}</Typography>
    </Box>
  );
}
