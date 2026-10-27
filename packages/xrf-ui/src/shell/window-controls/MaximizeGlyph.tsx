import { ReactElement } from "react";

import { CaptionGlyph } from "./CaptionGlyph";

export function MaximizeGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <rect x={"0.5"} y={"0.5"} width={"9"} height={"9"} />
    </CaptionGlyph>
  );
}
