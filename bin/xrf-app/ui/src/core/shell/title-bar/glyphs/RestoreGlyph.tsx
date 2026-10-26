import { ReactElement } from "react";

import { CaptionGlyph } from "@/core/shell/title-bar/glyphs/CaptionGlyph";

/** Two offset squares, the same way the system draws a window that can be put back where it was. */
export function RestoreGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M2.5 2.5V0.5h7v7h-2"} />
      <rect x={"0.5"} y={"2.5"} width={"7"} height={"7"} />
    </CaptionGlyph>
  );
}
