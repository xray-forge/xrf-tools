import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnHeaderChunk } from "@/core/ipc/types/xrf-db";
import { SpawnFileService } from "@/core/spawn/services";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { SpawnEditorHeaderTable } from "./SpawnEditorHeaderTable";

export function SpawnEditorHeader({
  "data-testid": dataTestId = "spawn-editor-header",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  return (
    <SpawnChunkView<SpawnHeaderChunk>
      data-testid={dataTestId}
      id={id}
      className={className}
      chunk={spawnFileService.chunks.header}
      render={(header: SpawnHeaderChunk) => <SpawnEditorHeaderTable header={header} />}
    />
  );
}
