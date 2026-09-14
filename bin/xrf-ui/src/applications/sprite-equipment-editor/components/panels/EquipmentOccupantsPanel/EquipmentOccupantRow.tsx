import { Box, Chip, ListItemButton, Typography } from "@mui/material";
import { ReactElement } from "react";

import { EEquipmentSlotClaim, EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
      className={className}
      selected={isSelected}
      sx={{ display: "block", paddingY: 0.5 }}
      onClick={onReveal}
    >
      <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
        <Typography variant={"body2"} sx={{ wordBreak: "break-all", flexGrow: 1 }}>
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
      </Box>

      {occupant.origin ? (
        <Typography variant={"caption"} color={"text.secondary"} sx={{ wordBreak: "break-all" }}>
          {occupant.origin}
        </Typography>
      ) : null}
    </ListItemButton>
  );
}
