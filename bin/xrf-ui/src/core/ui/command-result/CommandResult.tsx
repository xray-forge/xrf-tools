import { Box, Divider, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export type TCommandResultTone = "success" | "warning" | "error" | "info";

const TONE_COLORS: Record<TCommandResultTone, string> = {
  success: "success.main",
  warning: "warning.main",
  error: "error.main",
  info: "text.primary",
};

export interface ICommandResultStat {
  label: string;
  value: ReactNode;
  tone?: TCommandResultTone;
}

export interface ICommandResultProps extends BaseComponentProps {
  /** The one sentence answer to "how did it go". */
  headline: string;
  tone: TCommandResultTone;
  stats: Array<ICommandResultStat>;
  /** Ways to act on what was produced, shown beside the headline. */
  actions?: ReactNode;
  children?: ReactNode;
}

/**
 * Shared presentation for whatever a long running command produced.
 *
 * Tone comes from the palette, never a literal colour. The previous components printed their error
 * headings in hardcoded `green`.
 */
export function CommandResult({
  "data-testid": dataTestId,
  id,
  className,
  sx,
  headline,
  tone,
  stats,
  actions,
  children,
}: ICommandResultProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={[
        { display: "flex", flexDirection: "column", width: "100%", flexGrow: 1, minHeight: 0 },
        ...(sx === undefined ? [] : Array.isArray(sx) ? sx : [sx]),
      ]}
    >
      <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 2 }}>
        <Typography variant={"subtitle2"} sx={{ color: TONE_COLORS[tone] }}>
          {headline}
        </Typography>

        {actions ? <Box sx={{ display: "flex", flexShrink: 0, gap: 1 }}>{actions}</Box> : null}
      </Box>

      <Box sx={{ display: "flex", flexWrap: "wrap", columnGap: 3, rowGap: 0.5, marginTop: 1 }}>
        {stats.map((stat: ICommandResultStat) => (
          <Box key={stat.label} sx={{ display: "flex", alignItems: "baseline", gap: 0.75 }}>
            <Typography variant={"body2"} sx={{ color: stat.tone ? TONE_COLORS[stat.tone] : "text.primary" }}>
              {stat.value}
            </Typography>

            <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
              {stat.label}
            </Typography>
          </Box>
        ))}
      </Box>

      {children ? (
        <>
          <Divider sx={{ marginY: 2 }} />
          <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minHeight: 0 }}>{children}</Box>
        </>
      ) : null}
    </Box>
  );
}
