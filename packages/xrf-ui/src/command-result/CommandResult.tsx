import { formatCount } from "@xrf/format";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { Divider, Typography } from "#/material";
import { cn } from "#/theme/class-name";

export type TCommandResultTone = "success" | "warning" | "error" | "info";

export interface ICommandResultStat {
  label: string;
  /** What the stat measured; a number is a count, spelled through `formatCount` here so no caller forgets to. */
  value: ReactNode;
  tone?: TCommandResultTone;
}

const TONE_COLORS: Record<TCommandResultTone, string> = {
  success: "success.main",
  warning: "warning.main",
  error: "error.main",
  info: "text.primary",
};

interface ICommandResultProps extends BaseComponentProps {
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
  headline,
  tone,
  stats,
  actions,
  children,
}: ICommandResultProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 w-full grow flex-col", className)}>
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
              {typeof stat.value === "number" ? formatCount(stat.value) : stat.value}
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
    </div>
  );
}
