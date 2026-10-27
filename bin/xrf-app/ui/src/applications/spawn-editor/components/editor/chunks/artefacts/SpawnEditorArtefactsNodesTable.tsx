import { ArtefactSpawnPoint } from "@xrf/ipc/types/xrf-spawn";
import { GridColDef, GridRowId } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { decimalColumn, textColumn, vectorColumn } from "@xrf/ui/table";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";

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
