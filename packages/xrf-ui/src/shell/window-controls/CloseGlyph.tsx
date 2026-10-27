import { ReactElement } from "react";

import { CaptionGlyph } from "./CaptionGlyph";

export function CloseGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M0.5 0.5l9 9m0-9l-9 9"} />
    </CaptionGlyph>
  );
}
