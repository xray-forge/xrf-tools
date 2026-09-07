import { default as ViewListIcon } from "@mui/icons-material/ViewList";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";
import { Route, Routes } from "react-router-dom";

import { SpawnEditorAlife } from "@/applications/spawn-editor/components/editor/chunks/alife/SpawnEditorAlife";
import { SpawnEditorArtefacts } from "@/applications/spawn-editor/components/editor/chunks/artefacts/SpawnEditorArtefacts";
import { SpawnEditorGraphs } from "@/applications/spawn-editor/components/editor/chunks/graph/SpawnEditorGraphs";
import { SpawnEditorHeader } from "@/applications/spawn-editor/components/editor/chunks/header/SpawnEditorHeader";
import { SpawnEditorPatrols } from "@/applications/spawn-editor/components/editor/chunks/patrol/SpawnEditorPatrols";
import { createSpawnEditorPanels } from "@/applications/spawn-editor/components/editor/spawn-panels";
import { SpawnEditorActions } from "@/applications/spawn-editor/components/editor/SpawnEditorActions";
import { SpawnEditorMenu } from "@/applications/spawn-editor/components/editor/SpawnEditorMenu";
import { SpawnHeaderChunk } from "@/core/bindings/types/xrf-db";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { useEditorBusy } from "@/core/shell/EditorBusyContext";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { useEditorPanels } from "@/core/shell/panel/context";
import { SpawnFileService } from "@/core/spawn/services";
import { Nullable } from "@/lib/types/general";

export function SpawnEditor(): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const header: Nullable<SpawnHeaderChunk> = spawnFileService.header.value;
  const path: Nullable<string> = spawnFileService.path;

  useEditorPanels(
    () => [
      {
        icon: <ViewListIcon />,
        id: "chunks",
        isOpenByDefault: true,
        label: "Chunks",
        render: () => <SpawnEditorMenu />,
        side: "left",
      },
      ...createSpawnEditorPanels(spawnFileService),
    ],
    [spawnFileService]
  );

  // Closing does not navigate: the application shows its own picker again once nothing is open.
  const onClose = useCallback(() => spawnFileService.closeFile(), [spawnFileService]);

  useEditorBusy(spawnFileService.isBusy);

  useEditorStatus(
    header ? [`version ${header.version}`, `${header.objectsCount} objects`, `${header.levelsCount} levels`] : []
  );

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          actions={<SpawnEditorActions />}
          subtitle={path ? <EditorToolbarLocation location={{ path }} /> : null}
          onBack={onClose}
        />
      }
    >
      <Routes>
        <Route path={"/header"} element={<SpawnEditorHeader />} />
        <Route path={"/alife"} element={<SpawnEditorAlife />} />
        <Route path={"/artefacts"} element={<SpawnEditorArtefacts />} />
        <Route path={"/patrols/*"} element={<SpawnEditorPatrols />} />
        <Route path={"/graph/*"} element={<SpawnEditorGraphs />} />
        <Route path={"/*"} element={<SpawnEditorHeader />} />
      </Routes>
    </EditorLayout>
  );
}
