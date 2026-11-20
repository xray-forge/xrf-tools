import { useInjection } from "@wirestate/react";
import { SpawnALifeSpawnsChunk } from "@xrf/ipc/types/xrf-spawn";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/tools/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorAlifeObjectsTable } from "./SpawnEditorAlifeObjectsTable";

export function SpawnEditorAlife({
  "data-testid": dataTestId = "spawn-editor-alife",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnEditorService: SpawnEditorService = useInjection(SpawnEditorService);

  return (
    <SpawnChunkView<SpawnALifeSpawnsChunk>
      data-testid={dataTestId}
      id={id}
      className={className}
      chunk={spawnEditorService.chunks.alifeSpawn}
      render={(chunk: SpawnALifeSpawnsChunk) => <SpawnEditorAlifeObjectsTable objects={chunk.objects} />}
      onLoad={spawnEditorService.loadAlifeSpawn}
    />
  );
}
