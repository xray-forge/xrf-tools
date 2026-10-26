import { Box } from "@mui/material";
import { Fragment, ReactNode } from "react";

/**
 * Renders one help string, with backticked spans as code.
 *
 * @param text - Help string, possibly holding backticked spans.
 * @returns The string with each backticked span rendered as code.
 */
export function renderHelpText(text: string): ReactNode {
  const segments: Array<string> = text.split("`");

  if (segments.length === 1) {
    return text;
  }

  return segments.map((segment: string, index: number) =>
    index % 2 ? (
      <Box
        key={index}
        component={"code"}
        className={"rounded-control bg-action-hover px-1 font-monospace"}
        sx={{ fontSize: "0.8125em" }}
      >
        {segment}
      </Box>
    ) : (
      <Fragment key={index}>{segment}</Fragment>
    )
  );
}
