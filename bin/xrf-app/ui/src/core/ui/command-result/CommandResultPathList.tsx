import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { pathColumn } from "@/core/ui/table/columns";

interface ICommandResultPathListProps {
  /** Every path the run names, each its own row, id and search text. */
  paths: ReadonlyArray<string>;
  /** The column header over them. */
  column: string;
  emptyLabel: string;
  searchPlaceholder: string;
}

/** The paths a command produced or found, as one searchable column. */
export function CommandResultPathList({
  paths,
  column,
  emptyLabel,
  searchPlaceholder,
}: ICommandResultPathListProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(() => [pathColumn("path", column)], [column]);
  const rows: Array<{ path: string }> = useMemo(() => paths.map((path: string) => ({ path })), [paths]);

  return (
    <CommandResultFindings<{ path: string }>
      rows={rows}
      columns={columns}
      getRowId={(row) => row.path}
      getSearchText={(row) => row.path}
      emptyLabel={emptyLabel}
      searchPlaceholder={searchPlaceholder}
    />
  );
}
