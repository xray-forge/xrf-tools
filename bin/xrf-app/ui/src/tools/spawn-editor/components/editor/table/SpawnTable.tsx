import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { GridColDef, GridRowId, GridValidRowModel } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DataTable } from "@xrf/ui/table";
import { ReactElement, useCallback } from "react";

import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

interface ISpawnTableProps<T extends GridValidRowModel> extends BaseComponentProps {
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
  "data-testid": dataTestId = "spawn-table",
  id,
  className,
  source,
  rows,
  columns,
  getRowId,
  getSearchText,
  emptyLabel,
  countNoun,
  hiddenColumns,
}: ISpawnTableProps<T>): ReactElement {
  const spawnEditorService: SpawnEditorService = useInjection(SpawnEditorService);

  const selectedRowId: Nullable<GridRowId> = spawnEditorService.selectedRow?.id ?? null;

  const onRowSelect = useCallback(
    (row: T) => spawnEditorService.selectRow(source, getRowId(row), row),
    [source, spawnEditorService, getRowId]
  );

  return (
    <DataTable<T>
      data-testid={dataTestId}
      id={id}
      className={className}
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
