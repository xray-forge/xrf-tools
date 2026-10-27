import { useInjection } from "@wirestate/react";
import { SpawnHeaderChunk } from "@xrf/ipc/types/xrf-spawn";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnFileService } from "@/applications/spawn-editor/services/spawn-file.service";

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
