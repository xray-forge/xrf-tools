import { Box, Chip, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { LtxSectionSchemeReport } from "@/core/ipc/types/xrf-ltx-inspect";
import { MONOSPACE } from "@/core/theme/tokens";

interface IConfigsSchemeBindingProps {
  report: LtxSectionSchemeReport;
}

/**
 * The section, and the rule judging it stated in three facts.
 */
export function ConfigsSchemeBinding({ report }: IConfigsSchemeBindingProps): ReactElement {
  return (
    <Box sx={{ padding: 1.5, display: "flex", flexDirection: "column", gap: 0.75 }}>
      <Typography variant={"subtitle2"} sx={{ ...MONOSPACE, overflowWrap: "anywhere" }}>
        [{report.section}]
      </Typography>

      <Stack direction={"row"} sx={{ flexWrap: "wrap", gap: 0.5 }}>
        {report.scheme ? (
          <Chip
            size={"small"}
            variant={"outlined"}
            color={report.isDeclared ? "default" : "error"}
            label={report.scheme}
            title={report.isDeclared ? "Declared by a scheme file" : "No scheme file declares this name"}
          />
        ) : (
          <Chip size={"small"} variant={"outlined"} label={"no scheme"} title={"This section binds no scheme"} />
        )}

        {report.inheritedFrom ? (
          <Chip
            size={"small"}
            variant={"outlined"}
            label={`inherited from ${report.inheritedFrom}`}
            title={"The binding is written in that section, not in this one"}
          />
        ) : null}

        {report.isStrict ? (
          <Chip
            size={"small"}
            variant={"outlined"}
            label={"strict"}
            title={"Refuses fields it does not declare, and demands the ones it does not mark optional"}
          />
        ) : null}
      </Stack>
    </Box>
  );
}
