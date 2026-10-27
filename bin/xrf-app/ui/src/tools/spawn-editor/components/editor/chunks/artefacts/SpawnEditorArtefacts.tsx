import { useInjection } from "@wirestate/react";
import { SpawnArtefactSpawnsChunk } from "@xrf/ipc/types/xrf-spawn";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/tools/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnFileService } from "@/tools/spawn-editor/services/spawn-file.service";

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
