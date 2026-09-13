import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { ArtefactSpawnPoint } from "@/core/ipc/types/xrf-db";
import { decimalColumn, textColumn, vectorColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArtefactNodeRow extends ArtefactSpawnPoint {
  index: number;
}

interface ISpawnEditorArtefactsNodesTableProps extends BaseComponentProps {
  nodes: Array<ArtefactSpawnPoint>;
}

export function SpawnEditorArtefactsNodesTable({
  "data-testid": dataTestId = "spawn-editor-artefacts-nodes-table",
  id,
  className,
  nodes,
}: ISpawnEditorArtefactsNodesTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      textColumn("index", "#", 70),
      textColumn("levelVertexId", "Level vertex", 130),
      decimalColumn("distance", "Distance"),
      vectorColumn("position", "Position"),
    ],
    []
  );

  const rows: Array<IArtefactNodeRow> = useMemo(
    () => nodes.map((it: ArtefactSpawnPoint, index: number) => ({ ...it, index })),
    [nodes]
  );

  return (
    <SpawnTable<IArtefactNodeRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      countNoun={"node"}
      emptyLabel={"This file spawns no artefacts."}
      rows={rows}
      source={"Artefact spawn node"}
      getRowId={(row: IArtefactNodeRow): GridRowId => row.index}
    />
  );
}
