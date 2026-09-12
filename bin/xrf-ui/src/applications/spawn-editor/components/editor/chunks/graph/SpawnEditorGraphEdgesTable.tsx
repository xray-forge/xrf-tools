import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { GraphEdge } from "@/core/bindings/types/xrf-db";
import { decimalColumn, textColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
