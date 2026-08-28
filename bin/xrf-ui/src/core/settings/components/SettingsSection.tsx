import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { Nullable } from "@/lib/types/general";

export interface ISettingsSectionProps {
  title: string;
  description: string;
  /** What the setting currently amounts to, stated opposite the title. */
  fact?: Nullable<string>;
  /** What the heading introduces. A section that only announces the rows beneath it has none. */
  children?: ReactNode;
}

/**
 * A titled setting: what it is, one sentence on why, and whatever it controls.
 */
export function SettingsSection({ title, description, fact = null, children }: ISettingsSectionProps): ReactElement {
  return (
    <Box>
      <Box sx={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 1 }}>
        <Typography variant={"subtitle2"} sx={{ color: "text.primary" }}>
          {title}
        </Typography>

        {fact ? (
          <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
            {fact}
          </Typography>
        ) : null}
      </Box>

      <Typography
        variant={"caption"}
        sx={{ display: "block", color: "text.secondary", marginBottom: children ? 1 : 0 }}
      >
        {description}
      </Typography>

      {children}
    </Box>
  );
}
