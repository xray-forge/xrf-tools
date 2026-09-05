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
}: ICommandResultFindingsProps<T>): ReactElement {
  return (
    <DataTable<T>
      columns={columns}
      countNoun={"finding"}
      emptyLabel={emptyLabel}
      getRowId={getRowId}
      getSearchText={getSearchText}
      rows={rows}
      searchPlaceholder={searchPlaceholder}
    />
  );
}
