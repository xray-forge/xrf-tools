import { Typography } from "@mui/material";
import { ReactElement } from "react";

interface IArchivePathTextProps {
  value: string;
}

/**
 * A filesystem path, shown in full.
 *
 * Paths are long and their tail is the part that identifies them, so they wrap rather than truncate.
 */
export function ArchivePathText({ value }: IArchivePathTextProps): ReactElement {
  return (
    <Typography variant={"body2"} className={"monospace"} sx={{ wordBreak: "break-all" }}>
      {value}
    </Typography>
  );
}
