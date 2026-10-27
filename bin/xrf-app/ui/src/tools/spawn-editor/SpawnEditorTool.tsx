import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { SpawnFileService } from "@/tools/spawn-editor/services/spawn-file.service";

import { SpawnEditor } from "./components/editor/SpawnEditor";
import { SpawnEditorOpenForm } from "./components/SpawnEditorOpenForm";

/** Picker until a spawn file is open, editor once it is. */
export function SpawnEditorTool(): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  return <SessionTool root={spawnFileService.file} openForm={<SpawnEditorOpenForm />} workspace={<SpawnEditor />} />;
}
