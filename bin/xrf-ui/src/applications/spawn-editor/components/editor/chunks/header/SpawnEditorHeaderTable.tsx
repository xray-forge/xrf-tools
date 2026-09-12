import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { SpawnHeaderChunk } from "@/core/bindings/types/xrf-db";
import { identifierColumn, textColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IHeaderRow extends SpawnHeaderChunk {
  id: string;
}

interface ISpawnEditorHeaderTableProps extends BaseComponentProps {
  header: SpawnHeaderChunk;
}

export function SpawnEditorHeaderTable({
  "data-testid": dataTestId = "spawn-editor-header-table",
  id,
  className,
  header,
}: ISpawnEditorHeaderTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      textColumn("version", "Version"),
      textColumn("objectsCount", "Objects", 110),
      // Was `levelCount`, which is not a field the header carries - the column read empty in every file.
      textColumn("levelsCount", "Levels", 110),
      identifierColumn("guid", "Guid", 260),
      identifierColumn("graphGuid", "Graph guid", 260),
    ],
    []
  );

  const rows: Array<IHeaderRow> = useMemo(() => [{ ...header, id: "header" }], [header]);

  return (
    <SpawnTable<IHeaderRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={rows}
      countNoun={"header"}
      emptyLabel={"This file has no header chunk."}
      source={"Spawn header"}
      getRowId={(row: IHeaderRow): GridRowId => row.id}
    />
  );
}
