import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement } from "react";

import { DataTable } from "@/core/ui/table";

interface ICommandResultFindingsProps<T> {
  rows: Array<T>;
  columns: Array<GridColDef>;
  getRowId: (row: T) => GridRowId;
  /** Everything about a row that a search should match, flattened into one string. */
  getSearchText: (row: T) => string;
  emptyLabel: string;
  searchPlaceholder?: string;
  /** Columns worth keeping but not worth the width by default; the columns panel still offers them. */
  hiddenColumns?: Array<string>;
}

/**
 * The findings a command produced, as a sortable and filterable table.
 *
 * Kept as its own name because "findings" is what the result surfaces call these, and it pins the noun
 * the count is phrased with. Everything else is the shared table.
 */
export function CommandResultFindings<T>({
  rows,
  columns,
  getRowId,
  getSearchText,
  emptyLabel,
  searchPlaceholder = "Filter findings",
  hiddenColumns,
}: ICommandResultFindingsProps<T>): ReactElement {
  return (
    <DataTable<T>
      columns={columns}
      countNoun={"finding"}
      emptyLabel={emptyLabel}
      getRowId={getRowId}
      getSearchText={getSearchText}
      hiddenColumns={hiddenColumns}
      rows={rows}
      searchPlaceholder={searchPlaceholder}
    />
  );
}
