import { default as ViewListIcon } from "@mui/icons-material/ViewList";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";
import { Route, Routes } from "react-router-dom";

import { SpawnHeaderChunk } from "@/core/bindings/types/xrf-db";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { SpawnFileService } from "@/core/spawn/services";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { SpawnEditorAlife } from "./chunks/alife/SpawnEditorAlife";
import { SpawnEditorArtefacts } from "./chunks/artefacts/SpawnEditorArtefacts";
import { SpawnEditorGraphs } from "./chunks/graph/SpawnEditorGraphs";
import { SpawnEditorHeader } from "./chunks/header/SpawnEditorHeader";
import { SpawnEditorPatrols } from "./chunks/patrol/SpawnEditorPatrols";
import { SPAWN_EDITOR_PANELS } from "./spawn-panels";
import { SpawnEditorActions } from "./SpawnEditorActions";
import { SpawnEditorMenu } from "./SpawnEditorMenu";

export function SpawnEditor({
  "data-testid": dataTestId = "spawn-editor",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const header: Nullable<SpawnHeaderChunk> = spawnFileService.chunks.header.value;
  const path: Nullable<string> = spawnFileService.path;

  // Closing does not navigate: the application shows its own picker again once nothing is open.
  const onClose = useCallback(() => spawnFileService.closeFile(), [spawnFileService]);

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
      ...SPAWN_EDITOR_PANELS,
    ],
    []
  );

  useEditorBusy(spawnFileService.isBusy);

  useEditorStatus(
    header ? [`version ${header.version}`, `${header.objectsCount} objects`, `${header.levelsCount} levels`] : []
  );

  return (
    <EditorLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      toolbar={
        <EditorToolbar
          actions={<SpawnEditorActions />}
          subtitle={path ? <EditorToolbarLocation location={{ path }} /> : null}
          onBack={onClose}
        />
      }
    >
      <Routes key={spawnFileService.sessionId}>
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
