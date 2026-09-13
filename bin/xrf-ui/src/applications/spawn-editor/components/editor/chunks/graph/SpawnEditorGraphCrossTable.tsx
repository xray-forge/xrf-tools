import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { GraphCrossTable } from "@/core/ipc/types/xrf-db";
import { identifierColumn, textColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ISpawnEditorGraphCrossTableProps extends BaseComponentProps {
  crossTables: Array<GraphCrossTable>;
}

export function SpawnEditorGraphCrossTable({
  "data-testid": dataTestId = "spawn-editor-graph-cross-table",
  id,
  className,
  crossTables,
}: ISpawnEditorGraphCrossTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("levelGuid", "Level guid", 260),
      identifierColumn("gameGuid", "Game guid", 260),
      textColumn("version", "Version", 100),
      textColumn("nodesCount", "Nodes", 110),
      textColumn("verticesCount", "Vertices", 110),
    ],
    []
  );

  return (
    <SpawnTable<GraphCrossTable>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      countNoun={"cross table"}
      emptyLabel={"This graph has no cross tables."}
      rows={crossTables}
      source={"Graph cross table"}
      getRowId={(row: GraphCrossTable): GridRowId => row.levelGuid}
      getSearchText={(row: GraphCrossTable): string => row.levelGuid}
    />
  );
}
