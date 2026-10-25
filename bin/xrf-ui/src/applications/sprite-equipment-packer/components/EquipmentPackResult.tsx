import { Chip, Grid } from "@mui/material";
import { formatDuration } from "@xrf/format";
import { EquipmentSpritePackResult } from "@xrf/ipc/types/xrf-texture";
import { ReactElement } from "react";

interface IEquipmentPackResultProps {
  result: EquipmentSpritePackResult;
}

export function EquipmentPackResult({ result }: IEquipmentPackResultProps): ReactElement {
  return (
    <div className={"flex max-w-135 flex-col p-4"}>
      <Grid className={"justify-center gap-2"} container={true}>
        <Chip variant={"outlined"} color={"success"} label={formatDuration(result.duration)} />
        <Chip
          variant={"outlined"}
          color={"success"}
          label={`${result.packedCount + result.skippedCount} files total`}
        />
      </Grid>

      <Grid className={"mt-2 justify-center gap-2 px-4 py-0"} container={true}>
        <Chip variant={"outlined"} label={`${result.packedCount} file(s) packed`} />
        <Chip variant={"outlined"} label={`${result.skippedCount} file(s) skipped`} />
        <Chip variant={"outlined"} label={`${result.savedWidth}x${result.savedHeight} sprite`} />
      </Grid>
    </div>
  );
}
