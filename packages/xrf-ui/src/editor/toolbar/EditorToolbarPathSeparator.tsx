import { ReactElement } from "react";

import { Typography } from "#/material/typography";

export function EditorToolbarPathSeparator(): ReactElement {
  return (
    <Typography aria-hidden={true} className={"text-text-disabled select-none"} variant={"body2"}>
      ›
    </Typography>
  );
}
