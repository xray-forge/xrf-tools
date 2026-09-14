import { default as CloseIcon } from "@mui/icons-material/Close";
import { Box, Card, Chip, Divider, Grid, Typography } from "@mui/material";
import { ReactElement } from "react";

import { EEquipmentSlotClaim, EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { IEquipmentLayout, TEquipmentCell } from "@/core/sprite-equipment/lib";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { stopPropagation } from "@/lib/dom/event";

interface IEquipmentGridDetailsProps extends BaseComponentProps {
  cell: TEquipmentCell;
  layout: IEquipmentLayout;
  onClose: () => void;
}

export function EquipmentGridDetails({
  "data-testid": dataTestId = "equipment-grid-details",
  id,
  className,
  layout,
  cell,
  onClose,
}: IEquipmentGridDetailsProps): ReactElement {
  const items: ReadonlyArray<EquipmentSlotOccupant> = layout.at(cell);

  const list = items.map((it: EquipmentSlotOccupant) => (
    <Box key={it.section} sx={{ display: "flex", flexDirection: "column", marginTop: "6px" }}>
      <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
        <Typography variant={"body2"} sx={{ wordBreak: "break-all" }}>
          {it.section}
        </Typography>

        {it.claim === EEquipmentSlotClaim.DECLARED ? (
          <Chip
            size={"small"}
            variant={"outlined"}
            color={"primary"}
            label={"icon"}
            title={"Declares $inventory_icon, so the packing tools act on it"}
          />
        ) : null}
      </Box>

      {it.origin ? (
        <Typography variant={"caption"} color={"text.secondary"} sx={{ wordBreak: "break-all" }}>
          {it.origin}
        </Typography>
      ) : null}
    </Box>
  ));

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      onMouseMove={stopPropagation}
      onWheel={stopPropagation}
      onMouseDown={stopPropagation}
      sx={{ position: "absolute", left: 4, top: 4, maxWidth: 300, minWidth: 160, maxHeight: "50%", overflow: "auto" }}
    >
      <Card>
        <Box sx={{ display: "flex", flexDirection: "column", padding: 1, margin: 0, width: "100%", gap: 0.5 }}>
          <Grid container sx={{ justifyContent: "space-between", alignItems: "center", marginBottom: 1 }}>
            <Typography variant={"h6"}>{`${cell[1]}:${cell[0]}`}</Typography>

            <EditorIconAction
              label={"Close cell details"}
              description={"Hide the selected sprite cell details"}
              icon={<CloseIcon />}
              onClick={onClose}
            />
          </Grid>

          <Divider />

          {list.length ? list : "Nothing occupies this cell"}
        </Box>
      </Card>
    </Box>
  );
}
