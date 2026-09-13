import { default as CloseIcon } from "@mui/icons-material/Close";
import { Box, Card, Divider, Grid, Typography } from "@mui/material";
import { ReactElement } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { IEquipmentLayout, IEquipmentSectionDescriptor, TEquipmentCell } from "@/core/sprite-equipment/lib";
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
  const items: ReadonlyArray<IEquipmentSectionDescriptor> = layout.at(cell);

  const list = items.map((it: IEquipmentSectionDescriptor, index: number) => (
    <Box key={index} sx={{ marginTop: "4px" }}>
      {it.section}
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

          {list.length ? list : "No sprites"}
        </Box>
      </Card>
    </Box>
  );
}
