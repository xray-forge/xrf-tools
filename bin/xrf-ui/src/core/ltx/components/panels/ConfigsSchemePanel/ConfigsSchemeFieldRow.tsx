import { Box, Chip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { LtxSchemeFieldReport } from "@/core/ipc/types/xrf-ltx-inspect";
import { describeResolvedFieldOrigin } from "@/core/ltx/lib/resolved";
import { MONOSPACE } from "@/core/theme/tokens";

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
    <Box sx={{ display: "flex", flexDirection: "column", gap: 0.25, minWidth: 0 }}>
      <Box sx={{ display: "flex", alignItems: "baseline", flexWrap: "wrap", gap: 0.75, minWidth: 0 }}>
        <Typography
          component={"span"}
          variant={"body2"}
          sx={{ ...MONOSPACE, color: isMissing ? "text.disabled" : "text.primary", overflowWrap: "anywhere" }}
        >
          {field.name}
        </Typography>

        {field.declared ? (
          <Typography component={"span"} variant={"caption"} sx={{ ...MONOSPACE, color: "text.secondary" }}>
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
      </Box>

      {field.resolved ? (
        <>
          <Typography variant={"body2"} sx={{ ...MONOSPACE, overflowWrap: "anywhere" }}>
            {field.resolved.value}
          </Typography>

          <Typography variant={"caption"} sx={{ color: "text.secondary", overflowWrap: "anywhere" }}>
            {describeResolvedFieldOrigin(field.resolved.origin, false)}
          </Typography>
        </>
      ) : (
        <Typography variant={"caption"} sx={{ color: isStrict ? "error.main" : "text.secondary" }}>
          {isStrict && !field.declared?.isOptional ? "required, and not supplied" : "not supplied"}
        </Typography>
      )}
    </Box>
  );
}
