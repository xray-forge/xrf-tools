import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

import { TEquipmentCell } from "@/core/sprite-equipment/lib";

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
    <div data-testid={dataTestId} id={id} className={cn("absolute bottom-1 left-1", className)}>
      {`${cell[0]}:${cell[1]}`}
    </div>
  );
}
