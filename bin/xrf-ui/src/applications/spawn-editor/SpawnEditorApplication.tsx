import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpawnEditor } from "@/applications/spawn-editor/components/editor/SpawnEditor";
import { SpawnEditorOpenForm } from "@/applications/spawn-editor/components/SpawnEditorOpenForm";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { SpawnFileService } from "@/core/spawn/services";

/** Picker until a spawn file is open, editor once it is. */
export function SpawnEditorApplication(): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  if (!spawnFileService.isReady) {
    return <ApplicationLoader />;
  }

  return spawnFileService.isOpen ? <SpawnEditor /> : <SpawnEditorOpenForm />;
}
