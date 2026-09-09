import { GridColDef, GridRowId, GridValidRowModel } from "@mui/x-data-grid";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { SpawnFileService } from "@/core/spawn/services";
import { DataTable } from "@/core/ui/table";
import { Nullable } from "@/lib/types/general";

interface ISpawnTableProps<T extends GridValidRowModel> {
  /** What one row is, for the details panel heading. */
  source: string;
  rows: Array<T>;
  columns: Array<GridColDef<T>>;
  getRowId: (row: T) => GridRowId;
  getSearchText?: (row: T) => string;
  emptyLabel: string;
  countNoun: string;
  hiddenColumns?: Array<string>;
}

/**
 * A spawn chunk table, wired to the details panel.
 */
export function SpawnTable<T extends GridValidRowModel>({
  source,
  rows,
  columns,
  getRowId,
  getSearchText,
  emptyLabel,
  countNoun,
  hiddenColumns,
}: ISpawnTableProps<T>): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const selectedRowId: Nullable<GridRowId> = spawnFileService.selectedRow?.id ?? null;

  const onRowSelect = useCallback(
    (row: T) => spawnFileService.selectRow(source, getRowId(row), row),
    [source, spawnFileService, getRowId]
  );

  return (
    <DataTable<T>
      columns={columns}
      countNoun={countNoun}
      emptyLabel={emptyLabel}
      getRowId={getRowId}
      getSearchText={getSearchText}
      hiddenColumns={hiddenColumns}
      rows={rows}
      selectedRowId={selectedRowId}
      onRowSelect={onRowSelect}
    />
  );
}
