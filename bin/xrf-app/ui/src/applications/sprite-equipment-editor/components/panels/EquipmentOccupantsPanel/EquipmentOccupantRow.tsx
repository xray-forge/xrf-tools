import { EEquipmentSlotClaim, EquipmentSlotOccupant } from "@xrf/ipc/types/xrf-texture";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip, ListItemButton, Typography } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

interface IEquipmentOccupantRowProps extends BaseComponentProps {
  occupant: EquipmentSlotOccupant;
  isSelected: boolean;
  onReveal: () => void;
}

/**
 * One occupant: what it is, whether the packing tools act on it, where it sits, and which config declared it.
 */
export function EquipmentOccupantRow({
  "data-testid": dataTestId = "equipment-occupant-row",
  id,
  className,
  occupant,
  isSelected,
  onReveal,
}: IEquipmentOccupantRowProps): ReactElement {
  return (
    <ListItemButton
      data-testid={dataTestId}
      id={id}
      className={cn("block py-1", className)}
      selected={isSelected}
      onClick={onReveal}
    >
      <div className={"flex items-center gap-1"}>
        <Typography className={"grow break-all"} variant={"body2"}>
          {occupant.section}
        </Typography>

        {occupant.claim === EEquipmentSlotClaim.DECLARED ? (
          <Chip
            size={"small"}
            variant={"outlined"}
            color={"primary"}
            label={"icon"}
            title={"Declares $inventory_icon, so the packing tools act on it"}
          />
        ) : null}

        <Typography variant={"caption"} color={"text.secondary"}>
          {`${occupant.x}:${occupant.y}`}
        </Typography>
      </div>

      {occupant.origin ? (
        <Typography className={"break-all"} variant={"caption"} color={"text.secondary"}>
          {occupant.origin}
        </Typography>
      ) : null}
    </ListItemButton>
  );
}
