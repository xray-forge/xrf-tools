import { GraphEdge } from "@xrf/ipc/types/xrf-spawn";
import { GridColDef, GridRowId } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { decimalColumn, textColumn } from "@xrf/ui/table";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/tools/spawn-editor/components/editor/table/SpawnTable";

interface IGraphEdgeRow extends GraphEdge {
  index: number;
}

interface ISpawnEditorGraphEdgesTableProps extends BaseComponentProps {
  edges: Array<GraphEdge>;
}

export function SpawnEditorGraphEdgesTable({
  "data-testid": dataTestId = "spawn-editor-graph-edges-table",
  id,
  className,
  edges,
}: ISpawnEditorGraphEdgesTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      textColumn("index", "#", 90),
      textColumn("gameVertexId", "Game vertex", 140),
      decimalColumn("distance", "Distance", 130),
    ],
    []
  );

  const rows: Array<IGraphEdgeRow> = useMemo(
    () => edges.map((it: GraphEdge, index: number) => ({ ...it, index })),
    [edges]
  );

  return (
    <SpawnTable<IGraphEdgeRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={rows}
      countNoun={"edge"}
      emptyLabel={"This graph has no edges."}
      source={"Graph edge"}
      getRowId={(row: IGraphEdgeRow): GridRowId => row.index}
    />
  );
}
