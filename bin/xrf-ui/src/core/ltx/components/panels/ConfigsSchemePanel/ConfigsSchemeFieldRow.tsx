import { Chip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { LtxSchemeFieldReport } from "@/core/ipc/types/xrf-ltx-inspect";
import { describeResolvedFieldOrigin } from "@/core/ltx/lib/resolved";
import { cn } from "@/lib/dom/dom-name";

interface IConfigsSchemeFieldRowProps {
  field: LtxSchemeFieldReport;
  /**  Whether a scheme is judging this section at all. */
  isJudged: boolean;
  /** Whether a missing field is a finding here, which is what strictness decides. */
  isStrict: boolean;
}

/**
 * One field: what the scheme asks for, and what the section answers.
 */
export function ConfigsSchemeFieldRow({ field, isJudged, isStrict }: IConfigsSchemeFieldRowProps): ReactElement {
  const isMissing: boolean = !field.resolved;

  return (
    <div className={"flex min-w-0 flex-col gap-0.5"}>
      <div className={"flex min-w-0 flex-wrap items-baseline gap-1.5"}>
        <Typography
          className={cn("monospace wrap-anywhere", isMissing ? "text-text-disabled" : "text-text-primary")}
          component={"span"}
          variant={"body2"}
        >
          {field.name}
        </Typography>

        {field.declared ? (
          <Typography className={"monospace text-text-secondary"} component={"span"} variant={"caption"}>
            {field.declared.dataType}
            {field.declared.isArray ? "[]" : ""}
            {field.declared.isOptional ? "?" : ""}
            {field.declared.isAny ? " (*)" : ""}
          </Typography>
        ) : isJudged ? (
          <Chip
            size={"small"}
            variant={"outlined"}
            color={isStrict ? "error" : "default"}
            label={"undeclared"}
            title={
              isStrict
                ? "This scheme is strict, so a field it does not declare is a finding"
                : "The scheme does not name this field, and does not refuse it either"
            }
          />
        ) : null}
      </div>

      {field.resolved ? (
        <>
          <Typography className={"monospace wrap-anywhere"} variant={"body2"}>
            {field.resolved.value}
          </Typography>

          <Typography className={"wrap-anywhere text-text-secondary"} variant={"caption"}>
            {describeResolvedFieldOrigin(field.resolved.origin, false)}
          </Typography>
        </>
      ) : (
        <Typography className={isStrict ? "text-error" : "text-text-secondary"} variant={"caption"}>
          {isStrict && !field.declared?.isOptional ? "required, and not supplied" : "not supplied"}
        </Typography>
      )}
    </div>
  );
}
