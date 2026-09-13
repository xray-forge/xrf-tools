import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface ISettingsStatProps extends BaseComponentProps {
  label: string;
  value: string;
  /** What the figure is of, where the label cannot carry it: a unit, a denominator, a count behind a total. */
  hint?: Nullable<string>;
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
  hint = null,
}: ISettingsStatProps): ReactElement {
  return (
    <Box data-testid={dataTestId} className={className} id={id} sx={{ minWidth: 104 }}>
      <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block" }}>
        {label}
      </Typography>

      <Typography variant={"body2"}>{value}</Typography>

      {hint ? (
        <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block", opacity: 0.7 }}>
          {hint}
        </Typography>
      ) : null}
    </Box>
  );
}
