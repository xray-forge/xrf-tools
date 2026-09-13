import { Box, Link, Typography } from "@mui/material";
import { ReactElement } from "react";

import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface ISettingsAboutRowProps extends BaseComponentProps {
  label: string;
  value: string;
  /** Where the value can be followed to, for the few that address something outside the application. */
  href?: Nullable<string>;
}

/**
 * One stated fact about this build or the machine running it.
 */
export function SettingsAboutRow({
  "data-testid": dataTestId = "settings-about-row",
  className,
  id,
  label,
  value,
  href = null,
}: ISettingsAboutRowProps): ReactElement {
  return (
    <Box data-testid={dataTestId} className={className} id={id} sx={{ display: "flex", gap: 1 }}>
      <Typography variant={"caption"} sx={{ minWidth: 96, opacity: 0.7 }}>
        {label}
      </Typography>

      {href ? (
        <Link
          variant={"caption"}
          href={href}
          target={"_blank"}
          rel={"noreferrer"}
          sx={{ ...MONOSPACE, wordBreak: "break-all" }}
        >
          {value}
        </Link>
      ) : (
        <Typography variant={"caption"} sx={{ ...MONOSPACE, wordBreak: "break-all" }}>
          {value}
        </Typography>
      )}
    </Box>
  );
}
