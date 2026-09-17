import { Chip, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { LtxSectionSchemeReport } from "@/core/ipc/types/xrf-ltx-inspect";

interface IConfigsSchemeBindingProps {
  report: LtxSectionSchemeReport;
}

/**
 * The section, and the rule judging it stated in three facts.
 */
export function ConfigsSchemeBinding({ report }: IConfigsSchemeBindingProps): ReactElement {
  return (
    <div className={"flex flex-col gap-1.5 p-3"}>
      <Typography className={"monospace wrap-anywhere"} variant={"subtitle2"}>
        [{report.section}]
      </Typography>

      <Stack className={"flex-wrap gap-1"} direction={"row"}>
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
    </div>
  );
}
