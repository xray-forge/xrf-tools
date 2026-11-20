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
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorAlife } from "./editor/chunks/alife/SpawnEditorAlife";
import { SpawnEditorArtefacts } from "./editor/chunks/artefacts/SpawnEditorArtefacts";
import { SpawnEditorGraphs } from "./editor/chunks/graph/SpawnEditorGraphs";
import { SpawnEditorHeader } from "./editor/chunks/header/SpawnEditorHeader";
import { SpawnEditorPatrols } from "./editor/chunks/patrol/SpawnEditorPatrols";
import { SPAWN_EDITOR_PANELS } from "./editor/spawn-panels";
import { SpawnEditorActions } from "./editor/SpawnEditorActions";
import { SpawnEditorMenu } from "./editor/SpawnEditorMenu";

export function SpawnEditorWorkspace({
  "data-testid": dataTestId = "spawn-editor-workspace",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnEditorService: SpawnEditorService = useInjection(SpawnEditorService);

  const header: Nullable<SpawnHeaderChunk> = spawnEditorService.chunks.header.value;
  const path: Nullable<string> = spawnEditorService.path;

  // Closing does not navigate: the tool shows its own picker again once nothing is open.
  const onClose = useCallback(() => spawnEditorService.close(), [spawnEditorService]);

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

  useToolBusy(spawnEditorService.isBusy);

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
      <Routes key={spawnEditorService.sessionId}>
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
