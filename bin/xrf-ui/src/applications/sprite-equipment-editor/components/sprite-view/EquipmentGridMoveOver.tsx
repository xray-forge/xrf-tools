import { Box } from "@mui/material";
import { ReactElement } from "react";

import { TEquipmentCell } from "@/core/sprite-equipment/lib";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEquipmentGridMoveOver extends BaseComponentProps {
  cell: TEquipmentCell;
}

export function EquipmentGridMoveOver({
  "data-testid": dataTestId = "equipment-grid-move-over",
  id,
  className,
  cell,
}: IEquipmentGridMoveOver): ReactElement {
  return (
    <Box data-testid={dataTestId} id={id} className={className} sx={{ position: "absolute", left: 4, bottom: 4 }}>
      {`${cell[0]}:${cell[1]}`}
    </Box>
  );
}
