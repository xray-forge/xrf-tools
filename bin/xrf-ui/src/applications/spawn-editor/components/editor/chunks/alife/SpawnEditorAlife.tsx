import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnALifeSpawnsChunk } from "@/core/ipc/types/xrf-db";
import { SpawnFileService } from "@/core/spawn/services";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
