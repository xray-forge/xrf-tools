import { default as CloseIcon } from "@mui/icons-material/Close";
import { Box, Card, Divider, Grid, IconButton, Typography } from "@mui/material";
import { ReactElement } from "react";

import { GridMapper, IEquipmentSectionDescriptor, TEquipmentCell } from "@/core/sprite-equipment";
import { stopPropagation } from "@/lib/dom/event";
import { Nullable } from "@/lib/types/general";

interface IEquipmentGridDetailsProps {
  cell: TEquipmentCell;
  gridMapper: GridMapper;
  onClose: () => void;
}

export function EquipmentGridDetails({ gridMapper, cell, onClose }: IEquipmentGridDetailsProps): ReactElement {
  const [row, column] = cell;
  const items: Nullable<Array<IEquipmentSectionDescriptor>> = gridMapper.grid[row][column] ?? null;

  const list = items?.map((it, index) => (
    <Box key={index} sx={{ marginTop: "4px" }}>
      {it.section}
    </Box>
  ));

  return (
    <Box
      onMouseMove={stopPropagation}
      onWheel={stopPropagation}
      onMouseDown={stopPropagation}
      sx={{ position: "absolute", left: 4, top: 4, maxWidth: 300, minWidth: 160, maxHeight: "50%", overflow: "auto" }}
    >
      <Card>
        <Box sx={{ display: "flex", flexDirection: "column", padding: 1, margin: 0, width: "100%", gap: 0.5 }}>
          <Grid container sx={{ justifyContent: "space-between", alignItems: "center", marginBottom: 1 }}>
            <Typography variant={"h6"}>{`${cell[1]}:${cell[0]}`}</Typography>

            <IconButton size={"small"} onClick={onClose}>
              <CloseIcon />
            </IconButton>
          </Grid>

          <Divider />

          {list?.length ? list : "No sprites"}
        </Box>
      </Card>
    </Box>
  );
}
