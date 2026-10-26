import { Typography } from "@mui/material";
import { ReactElement } from "react";

export function EditorToolbarPathSeparator(): ReactElement {
  return (
    <Typography aria-hidden={true} className={"text-text-disabled select-none"} variant={"body2"}>
      ›
    </Typography>
  );
}
