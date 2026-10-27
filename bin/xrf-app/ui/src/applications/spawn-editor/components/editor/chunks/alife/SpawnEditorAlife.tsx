import { useInjection } from "@wirestate/react";
import { SpawnALifeSpawnsChunk } from "@xrf/ipc/types/xrf-spawn";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnFileService } from "@/applications/spawn-editor/services/spawn-file.service";

import { SpawnEditorAlifeObjectsTable } from "./SpawnEditorAlifeObjectsTable";

export function SpawnEditorAlife({
  "data-testid": dataTestId = "spawn-editor-alife",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  return (
    <SpawnChunkView<SpawnALifeSpawnsChunk>
      data-testid={dataTestId}
      id={id}
      className={className}
      chunk={spawnFileService.chunks.alifeSpawn}
      render={(chunk: SpawnALifeSpawnsChunk) => <SpawnEditorAlifeObjectsTable objects={chunk.objects} />}
      onLoad={spawnFileService.loadAlifeSpawn}
    />
  );
}
