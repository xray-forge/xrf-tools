import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { SpawnFileService } from "@/core/spawn/services";

import { SpawnEditor } from "./components/editor/SpawnEditor";
import { SpawnEditorOpenForm } from "./components/SpawnEditorOpenForm";

/** Picker until a spawn file is open, editor once it is. */
export function SpawnEditorApplication(): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  if (!spawnFileService.isReady) {
    return <ApplicationLoader />;
  }

  return spawnFileService.isOpen ? <SpawnEditor /> : <SpawnEditorOpenForm />;
}
