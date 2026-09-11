import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpawnEditorHeaderTable } from "@/applications/spawn-editor/components/editor/chunks/header/SpawnEditorHeaderTable";
import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnHeaderChunk } from "@/core/bindings/types/xrf-db";
import { SpawnFileService } from "@/core/spawn/services";

export function SpawnEditorHeader(): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  return (
    <SpawnChunkView<SpawnHeaderChunk>
      chunk={spawnFileService.chunks.header}
      render={(header: SpawnHeaderChunk) => <SpawnEditorHeaderTable header={header} />}
    />
  );
}
