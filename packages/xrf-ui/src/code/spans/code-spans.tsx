import { Fragment, ReactNode } from "react";

import { Box } from "#/layout/stack";

/**
 * Renders a line of prose with its backticked spans as inline code, as help and descriptions write identifiers.
 *
 * @param text - Prose, possibly holding backticked spans.
 * @returns The text as it came when it holds no backtick, otherwise each backticked span rendered as code.
 */
export function renderCodeSpans(text: string): ReactNode {
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
