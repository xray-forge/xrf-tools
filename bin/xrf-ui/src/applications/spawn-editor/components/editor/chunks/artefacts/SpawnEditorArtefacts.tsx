import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpawnEditorArtefactsNodesTable } from "@/applications/spawn-editor/components/editor/chunks/artefacts/SpawnEditorArtefactsNodesTable";
import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnArtefactSpawnsChunk } from "@/core/bindings/types/xrf-db";
import { SpawnFileService } from "@/core/spawn/services";

export function SpawnEditorArtefacts(): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  return (
    <SpawnChunkView<SpawnArtefactSpawnsChunk>
      chunk={spawnFileService.chunks.artefactSpawn}
      render={(chunk: SpawnArtefactSpawnsChunk) => <SpawnEditorArtefactsNodesTable nodes={chunk.nodes} />}
      onLoad={spawnFileService.loadArtefactSpawn}
    />
  );
}
