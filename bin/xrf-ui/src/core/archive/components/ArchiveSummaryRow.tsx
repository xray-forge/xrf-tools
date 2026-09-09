import { Box, Stack, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

interface IArchiveSummaryRowProps {
  label: string;
  children: ReactNode;
}

/** One labelled line of a summary, with the labels in a column of their own. */
export function ArchiveSummaryRow({ label, children }: IArchiveSummaryRowProps): ReactElement {
  return (
    <Stack direction={"row"} spacing={2} sx={{ alignItems: "baseline" }}>
      <Typography variant={"caption"} sx={{ width: 116, flexShrink: 0, color: "text.secondary" }}>
        {label}
      </Typography>

      <Box sx={{ minWidth: 0, flexGrow: 1 }}>{children}</Box>
    </Stack>
  );
}
