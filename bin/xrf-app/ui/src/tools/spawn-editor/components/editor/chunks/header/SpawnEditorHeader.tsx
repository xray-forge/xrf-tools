import { useInjection } from "@wirestate/react";
import { SpawnHeaderChunk } from "@xrf/ipc/types/xrf-spawn";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/tools/spawn-editor/components/editor/chunks/SpawnChunkView";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorHeaderTable } from "./SpawnEditorHeaderTable";

export function SpawnEditorHeader({
  "data-testid": dataTestId = "spawn-editor-header",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnEditorService: SpawnEditorService = useInjection(SpawnEditorService);

  return (
    <SpawnChunkView<SpawnHeaderChunk>
      data-testid={dataTestId}
      id={id}
      className={className}
      chunk={spawnEditorService.chunks.header}
      render={(header: SpawnHeaderChunk) => <SpawnEditorHeaderTable header={header} />}
    />
  );
}
