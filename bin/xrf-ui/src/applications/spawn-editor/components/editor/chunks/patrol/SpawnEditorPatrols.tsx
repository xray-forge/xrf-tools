import { Tab, Tabs } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { TChunkTabChange, useChunkTab } from "@/applications/spawn-editor/components/editor/chunks/use-chunk-tab";
import { SpawnPatrolsChunk } from "@/core/ipc/types/xrf-spawn";
import { SpawnFileService } from "@/core/spawn/services";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { SpawnEditorPatrolLinksTable } from "./SpawnEditorPatrolLinksTable";
import { SpawnEditorPatrolPointsTable } from "./SpawnEditorPatrolPointsTable";
import { SpawnEditorPatrolsTable } from "./SpawnEditorPatrolsTable";

const BASE_PATH: string = "/spawn-editor/patrols";
const TABS: Array<string> = ["patrols", "points", "links"];

export function SpawnEditorPatrols({
  "data-testid": dataTestId = "spawn-editor-patrols",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const [activeTab, onChangeTab]: [string, TChunkTabChange] = useChunkTab(BASE_PATH, TABS, "patrols");

  return (
    <SpawnChunkView<SpawnPatrolsChunk>
      data-testid={dataTestId}
      id={id}
      className={className}
      chunk={spawnFileService.chunks.patrols}
      render={(chunk: SpawnPatrolsChunk) => (
        <>
          <Tabs className={"mb-2 shrink-0"} value={activeTab} onChange={onChangeTab}>
            <Tab value={"patrols"} label={"Patrols"} />
            <Tab value={"points"} label={"Points"} />
            <Tab value={"links"} label={"Links"} />
          </Tabs>

          <div className={"flex min-h-0 grow"}>
            {activeTab === "points" ? <SpawnEditorPatrolPointsTable patrols={chunk.patrols} /> : null}
            {activeTab === "links" ? <SpawnEditorPatrolLinksTable patrols={chunk.patrols} /> : null}
            {activeTab === "patrols" ? <SpawnEditorPatrolsTable patrols={chunk.patrols} /> : null}
          </div>
        </>
      )}
      onLoad={spawnFileService.loadPatrols}
    />
  );
}
