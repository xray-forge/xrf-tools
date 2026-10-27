import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { SpawnHeaderChunk } from "@xrf/ipc/types/xrf-spawn";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ViewListIcon } from "@xrf/ui/icons";
import { EditorLayout } from "@xrf/ui/shell/editor/EditorLayout";
import { EditorToolbarLocation } from "@xrf/ui/shell/editor/EditorToolbarLocation";
import { ReactElement, useCallback } from "react";
import { Route, Routes } from "react-router-dom";

import { SpawnFileService } from "@/applications/spawn-editor/services/spawn-file.service";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";

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
    header
      ? [
          `version ${header.version}`,
          `${formatCount(header.objectsCount)} objects`,
          `${formatCount(header.levelsCount)} levels`,
        ]
      : []
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
