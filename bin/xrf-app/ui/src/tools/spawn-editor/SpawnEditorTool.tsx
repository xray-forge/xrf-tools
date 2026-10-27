import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ToolLoader } from "@/core/shell/loading/ToolLoader";
import { SpawnFileService } from "@/tools/spawn-editor/services/spawn-file.service";

import { SpawnEditor } from "./components/editor/SpawnEditor";
import { SpawnEditorOpenForm } from "./components/SpawnEditorOpenForm";

/** Picker until a spawn file is open, editor once it is. */
export function SpawnEditorTool(): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  if (!spawnFileService.isReady) {
    return <ToolLoader />;
  }

  return spawnFileService.isOpen ? <SpawnEditor /> : <SpawnEditorOpenForm />;
}
