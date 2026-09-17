import { Stack, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveSummaryRowProps extends BaseComponentProps {
  label: string;
  children: ReactNode;
}

/** One labelled line of a summary, with the labels in a column of their own. */
export function ArchiveSummaryRow({
  "data-testid": dataTestId = "archive-summary-row",
  id,
  className,
  label,
  children,
}: IArchiveSummaryRowProps): ReactElement {
  return (
    <Stack data-testid={dataTestId} id={id} className={cn("items-baseline", className)} direction={"row"} spacing={2}>
      <Typography className={"w-29 shrink-0 text-text-secondary"} variant={"caption"}>
        {label}
      </Typography>

      <div className={"min-w-0 grow"}>{children}</div>
    </Stack>
  );
}
