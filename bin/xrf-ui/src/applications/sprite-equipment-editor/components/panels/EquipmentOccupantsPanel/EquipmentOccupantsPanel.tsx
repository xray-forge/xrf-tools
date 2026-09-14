import { Box, List, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { EquipmentGridService } from "@/applications/sprite-equipment-editor/services/grid";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { isSameCell, TEquipmentCell } from "@/core/sprite-equipment/lib";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { IEquipmentOccupantRow, toEquipmentOccupantRows } from "./equipment-occupant-rows";
import { EquipmentOccupantRow } from "./EquipmentOccupantRow";

/**
 * Every section the configuration puts on the sheet, as a jump list.
 */
export function EquipmentOccupantsPanel({
  "data-testid": dataTestId = "equipment-occupants-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const gridService: EquipmentGridService = useInjection(EquipmentGridService);

  const [query, setQuery] = useState<string>("");

  const selectedCell: Nullable<TEquipmentCell> = gridService.selectedCell;
  const rows: Array<IEquipmentOccupantRow> = useMemo(
    () => toEquipmentOccupantRows(gridService.layout, query),
    [gridService.layout, query]
  );

  const onClear = useCallback(() => setQuery(""), []);

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}
    >
      <EditorSearchHeader
        title={"Occupants"}
        count={rows.length}
        query={query}
        placeholder={"Filter by section or config"}
        ariaLabel={"Filter occupants"}
        onClear={onClear}
        onQueryChange={setQuery}
      />

      <List dense={true} sx={{ flexGrow: 1, minHeight: 0, overflow: "auto", paddingY: 0 }}>
        {rows.map((row: IEquipmentOccupantRow) => (
          <EquipmentOccupantRow
            key={row.occupant.section}
            occupant={row.occupant}
            isSelected={isSameCell(selectedCell, row.cell)}
            onReveal={() => gridService.revealCell(row.cell)}
          />
        ))}
      </List>

      {rows.length ? null : (
        <Typography variant={"caption"} color={"text.secondary"} sx={{ padding: 1 }}>
          {gridService.layout ? "Nothing matches" : "No sheet open"}
        </Typography>
      )}
    </Box>
  );
}
