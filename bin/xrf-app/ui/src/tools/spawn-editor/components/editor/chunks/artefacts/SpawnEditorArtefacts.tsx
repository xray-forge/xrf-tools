import { useInjection } from "@wirestate/react";
import { SpawnArtefactSpawnsChunk } from "@xrf/ipc/types/xrf-spawn";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/tools/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorArtefactsNodesTable } from "./SpawnEditorArtefactsNodesTable";

export function SpawnEditorArtefacts({
  "data-testid": dataTestId = "spawn-editor-artefacts",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnEditorService: SpawnEditorService = useInjection(SpawnEditorService);

  return (
    <SpawnChunkView<SpawnArtefactSpawnsChunk>
      data-testid={dataTestId}
      id={id}
      className={className}
      chunk={spawnEditorService.chunks.artefactSpawn}
      render={(chunk: SpawnArtefactSpawnsChunk) => <SpawnEditorArtefactsNodesTable nodes={chunk.nodes} />}
      onLoad={spawnEditorService.loadArtefactSpawn}
    />
  );
}
