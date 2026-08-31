import { Box, Chip, Grid } from "@mui/material";
import { ReactElement } from "react";

import { IPackEquipmentResult } from "@/core/sprite-equipment";
import { formatDuration } from "@/lib/format/duration";

interface IEquipmentPackResultProps {
  result: IPackEquipmentResult;
}

export function EquipmentPackResult({ result }: IEquipmentPackResultProps): ReactElement {
  return (
    <Box sx={{ display: "flex", flexDirection: "column", padding: 2, maxWidth: 540 }}>
      <Grid container sx={{ justifyContent: "center", gap: 1 }}>
        <Chip variant={"outlined"} color={"success"} label={formatDuration(result.duration)} />
        <Chip
          variant={"outlined"}
          color={"success"}
          label={`${result.packedCount + result.skippedCount} files total`}
        />
      </Grid>

      <Grid container sx={{ justifyContent: "center", gap: 1, marginTop: 1, padding: `0 ${16}px` }}>
        <Chip variant={"outlined"} label={`${result.packedCount} file(s) packed`} />
        <Chip variant={"outlined"} label={`${result.skippedCount} file(s) skipped`} />
        <Chip variant={"outlined"} label={`${result.savedWidth}x${result.savedHeight} sprite`} />
      </Grid>
    </Box>
  );
}
