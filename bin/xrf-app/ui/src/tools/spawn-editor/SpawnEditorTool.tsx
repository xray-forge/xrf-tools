import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorOpenForm } from "./components/SpawnEditorOpenForm";
import { SpawnEditorWorkspace } from "./components/SpawnEditorWorkspace";

/** Picker until a spawn file is open, editor once it is. */
export function SpawnEditorTool(): ReactElement {
  const spawnEditorService: SpawnEditorService = useInjection(SpawnEditorService);

  return (
    <SessionTool
      root={spawnEditorService.file}
      openForm={<SpawnEditorOpenForm />}
      workspace={<SpawnEditorWorkspace />}
    />
  );
}
