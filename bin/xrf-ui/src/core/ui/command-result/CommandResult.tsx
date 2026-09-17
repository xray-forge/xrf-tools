import { Box, Divider, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { StyledComponentProps } from "@/lib/dom/element-types";

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

interface ICommandResultProps extends StyledComponentProps {
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
    <Box data-testid={dataTestId} id={id} className={cn("flex min-h-0 w-full grow flex-col", className)} sx={sx}>
      <div className={"flex items-center justify-between gap-4"}>
        <Typography variant={"subtitle2"} sx={{ color: TONE_COLORS[tone] }}>
          {headline}
        </Typography>

        {actions ? <div className={"flex shrink-0 gap-2"}>{actions}</div> : null}
      </div>

      <div className={"mt-2 flex flex-wrap gap-x-6 gap-y-1"}>
        {stats.map((stat: ICommandResultStat) => (
          <div key={stat.label} className={"flex items-baseline gap-1.5"}>
            <Typography variant={"body2"} sx={{ color: stat.tone ? TONE_COLORS[stat.tone] : "text.primary" }}>
              {stat.value}
            </Typography>

            <Typography className={"text-text-secondary"} variant={"caption"}>
              {stat.label}
            </Typography>
          </div>
        ))}
      </div>

      {children ? (
        <>
          <Divider className={"my-4"} />
          <div className={"flex min-h-0 grow flex-col"}>{children}</div>
        </>
      ) : null}
    </Box>
  );
}
