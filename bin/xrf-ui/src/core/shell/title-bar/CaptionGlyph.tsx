import { Box } from "@mui/material";
import { ReactElement, ReactNode } from "react";

interface ICaptionGlyphProps {
  children: ReactNode;
}

/**
 * Windows draws its caption glyphs on a 10x10 box with a hairline stroke, so these do too.
 *
 * Drawn rather than taken from an icon set: the icon fonts round their corners and thicken their
 * strokes, which reads as an application's own button instead of the window's.
 */
export function CaptionGlyph({ children }: ICaptionGlyphProps): ReactElement {
  return (
    <Box
      component={"svg"}
      viewBox={"0 0 10 10"}
      aria-hidden={true}
      sx={{ width: 10, height: 10, fill: "none", stroke: "currentColor", strokeWidth: 1 }}
    >
      {children}
    </Box>
  );
}

export function MinimizeGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M0 5h10"} />
    </CaptionGlyph>
  );
}

export function MaximizeGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <rect x={"0.5"} y={"0.5"} width={"9"} height={"9"} />
    </CaptionGlyph>
  );
}

/** Two offset squares, the same way the system draws a window that can be put back where it was. */
export function RestoreGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M2.5 2.5V0.5h7v7h-2"} />
      <rect x={"0.5"} y={"2.5"} width={"7"} height={"7"} />
    </CaptionGlyph>
  );
}

export function CloseGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M0.5 0.5l9 9m0-9l-9 9"} />
    </CaptionGlyph>
  );
}
