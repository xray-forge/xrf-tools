import { Tab, Tabs } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpawnChunkView } from "@/applications/spawn-editor/components/editor/chunks/SpawnChunkView";
import { TChunkTabChange, useChunkTab } from "@/applications/spawn-editor/components/editor/chunks/use-chunk-tab";
import { SpawnGraphsChunk } from "@/core/ipc/types/xrf-db";
import { SpawnFileService } from "@/core/spawn/services";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { SpawnEditorGraphCrossTable } from "./SpawnEditorGraphCrossTable";
import { SpawnEditorGraphEdgesTable } from "./SpawnEditorGraphEdgesTable";
import { SpawnEditorGraphHeaderTable } from "./SpawnEditorGraphHeaderTable";
import { SpawnEditorGraphLevelsTable } from "./SpawnEditorGraphLevelsTable";
import { SpawnEditorGraphPointsTable } from "./SpawnEditorGraphPointsTable";
import { SpawnEditorGraphVerticesTable } from "./SpawnEditorGraphVerticesTable";

const BASE_PATH: string = "/spawn-editor/graph";
const TABS: Array<string> = ["header", "levels", "vertices", "edges", "points", "cross-tables"];

export function SpawnEditorGraphs({
  "data-testid": dataTestId = "spawn-editor-graphs",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const [activeTab, onChangeTab]: [string, TChunkTabChange] = useChunkTab(BASE_PATH, TABS, "header");

  return (
    <SpawnChunkView<SpawnGraphsChunk>
      data-testid={dataTestId}
      id={id}
      className={className}
      chunk={spawnFileService.chunks.graphs}
      render={(chunk: SpawnGraphsChunk) => (
        <>
          <Tabs className={"mb-2 shrink-0"} value={activeTab} variant={"scrollable"} onChange={onChangeTab}>
            <Tab value={"header"} label={"Header"} />
            <Tab value={"levels"} label={"Levels"} />
            <Tab value={"vertices"} label={"Vertices"} />
            <Tab value={"edges"} label={"Edges"} />
            <Tab value={"points"} label={"Points"} />
            <Tab value={"cross-tables"} label={"Cross tables"} />
          </Tabs>

          <div className={"flex min-h-0 grow"}>
            {activeTab === "levels" ? <SpawnEditorGraphLevelsTable levels={chunk.levels} /> : null}
            {activeTab === "vertices" ? <SpawnEditorGraphVerticesTable vertices={chunk.vertices} /> : null}
            {activeTab === "edges" ? <SpawnEditorGraphEdgesTable edges={chunk.edges} /> : null}
            {activeTab === "points" ? <SpawnEditorGraphPointsTable points={chunk.points} /> : null}
            {activeTab === "cross-tables" ? <SpawnEditorGraphCrossTable crossTables={chunk.crossTables} /> : null}
            {activeTab === "header" ? <SpawnEditorGraphHeaderTable header={chunk.header} /> : null}
          </div>
        </>
      )}
      onLoad={spawnFileService.loadGraphs}
    />
  );
}
