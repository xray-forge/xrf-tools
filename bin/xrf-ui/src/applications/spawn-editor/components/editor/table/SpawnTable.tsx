import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { SpawnFileService } from "@/core/spawn/services";
import { DataTable } from "@/core/ui/table";
import { AnyObject, Nullable } from "@/lib/types/general";

interface ISpawnTableProps<T> {
  /** What one row is, for the details panel heading. */
  source: string;
  rows: Array<T>;
  columns: Array<GridColDef>;
  getRowId: (row: T) => GridRowId;
  getSearchText?: (row: T) => string;
  emptyLabel: string;
  countNoun: string;
  hiddenColumns?: Array<string>;
}

/**
 * A spawn chunk table, wired to the details panel.
 */
export function SpawnTable<T>({
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
    (row: T) => spawnFileService.selectRow(source, getRowId(row), row as AnyObject),
    // `getRowId` is declared inline by every caller, so depending on it would rebuild this per render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [source, spawnFileService]
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
