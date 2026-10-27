import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { SpawnHeaderChunk } from "@xrf/ipc/types/xrf-spawn";
import { Nullable } from "@xrf/types";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { EditorToolbarLocation } from "@xrf/ui/editor/toolbar";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ViewListIcon } from "@xrf/ui/icons/view-list";
import { ReactElement, useCallback } from "react";
import { Route, Routes } from "react-router-dom";

import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";
import { useShellPanels, useShellStatus } from "@/core/shell/contributions";
import { useToolBusy } from "@/core/shell/lifecycle";
import { SpawnFileService } from "@/tools/spawn-editor/services/spawn-file.service";

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

  // Closing does not navigate: the tool shows its own picker again once nothing is open.
  const onClose = useCallback(() => spawnFileService.closeFile(), [spawnFileService]);

  useShellPanels(
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

  useToolBusy(spawnFileService.isBusy);

  useShellStatus(
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
