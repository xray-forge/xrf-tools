import { Box } from "@mui/material";
import { ReactElement } from "react";

import { TEquipmentCell } from "@/core/sprite-equipment";

interface IEquipmentGridMoveOver {
  cell: TEquipmentCell;
}

export function EquipmentGridMoveOver({ cell }: IEquipmentGridMoveOver): ReactElement {
  return <Box sx={{ position: "absolute", left: 4, bottom: 4 }}>{`${cell[0]}:${cell[1]}`}</Box>;
}
