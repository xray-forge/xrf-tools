import { ReactElement } from "react";

import { CaptionGlyph } from "@/core/shell/title-bar/glyphs/CaptionGlyph";

export function MinimizeGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M0 5h10"} />
    </CaptionGlyph>
  );
}
