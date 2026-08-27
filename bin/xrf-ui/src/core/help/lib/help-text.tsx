import { Box } from "@mui/material";
import { Fragment, ReactNode } from "react";

import { RADIUS } from "@/core/theme/tokens";

export const HELP_MONOSPACE_FONT: string = "'Cascadia Mono', 'Consolas', monospace";

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
        sx={{
          paddingX: 0.5,
          borderRadius: `${RADIUS.sm}px`,
          backgroundColor: "action.hover",
          fontFamily: HELP_MONOSPACE_FONT,
          fontSize: "0.8125em",
        }}
      >
        {segment}
      </Box>
    ) : (
      <Fragment key={index}>{segment}</Fragment>
    )
  );
}
