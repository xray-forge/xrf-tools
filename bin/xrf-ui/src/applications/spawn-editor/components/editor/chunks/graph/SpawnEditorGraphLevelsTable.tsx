import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { GraphLevel } from "@/core/bindings/types/xrf-db";
import { identifierColumn, textColumn, vectorColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ISpawnEditorGraphLevelsTableProps extends BaseComponentProps {
  levels: Array<GraphLevel>;
}

export function SpawnEditorGraphLevelsTable({
  "data-testid": dataTestId = "spawn-editor-graph-levels-table",
  id,
  className,
  levels,
}: ISpawnEditorGraphLevelsTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      textColumn("id", "Id", 80),
      identifierColumn("name", "Name", 200),
      identifierColumn("section", "Section", 180),
      vectorColumn("offset", "Offset"),
      identifierColumn("guid", "Guid", 260),
    ],
    []
  );

  return (
    <SpawnTable<GraphLevel>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={levels}
      countNoun={"level"}
      emptyLabel={"This graph has no levels."}
      source={"Graph level"}
      getRowId={(row: GraphLevel): GridRowId => row.id}
      getSearchText={(row: GraphLevel): string => `${row.name} ${row.section}`}
    />
  );
}
