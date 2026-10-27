import { useInjection } from "@wirestate/react";
import { SpawnArtefactSpawnsChunk } from "@xrf/ipc/types/xrf-spawn";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnFileService } from "@/applications/spawn-editor/services/spawn-file";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { SpawnEditorArtefactsNodesTable } from "./SpawnEditorArtefactsNodesTable";

export function SpawnEditorArtefacts({
  "data-testid": dataTestId = "spawn-editor-artefacts",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  return (
    <SpawnChunkView<SpawnArtefactSpawnsChunk>
      data-testid={dataTestId}
      id={id}
      className={className}
      chunk={spawnFileService.chunks.artefactSpawn}
      render={(chunk: SpawnArtefactSpawnsChunk) => <SpawnEditorArtefactsNodesTable nodes={chunk.nodes} />}
      onLoad={spawnFileService.loadArtefactSpawn}
    />
  );
}
