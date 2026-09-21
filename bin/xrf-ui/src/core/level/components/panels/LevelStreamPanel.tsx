import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import {
  ILevelStreamStages,
  ILevelStreamSummary,
  LEVEL_STREAM_STAGES,
} from "@/core/level/lib/stream/level-stream-profile";
import { ILevelTextureProblem, ILevelTextureReport } from "@/core/level/lib/surface/level-surface-dressing";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";
import { formatBytes } from "@/lib/memory/format";

/**
 * What the viewport is holding and what it costs, measured rather than estimated.
 */
export function LevelStreamPanel({
  "data-testid": dataTestId = "level-stream-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const stats: ILevelStats = viewportService.stats;
  const textures: ILevelTextureReport = viewportService.textureReport;
  const problems: ReadonlyArray<ILevelTextureProblem> = textures.problems;
  const stream: ILevelStreamSummary = loadService.streamProfile;

  if (!loadService.level.value) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Streaming"}>
        <EditorPanelEmpty label={"No level open. Open one to see what it costs to draw."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Streaming"}>
      <EditorPanelSection title={"Resident"} isFirst>
        <EditorPanelProperty label={"Sectors"} value={stats.sectors} />
        <EditorPanelProperty label={"Geometry"} value={formatBytes(stats.bytes)} />
      </EditorPanelSection>

      {stream.mean ? (
        <EditorPanelSection title={"Reading a sector"}>
          {LEVEL_STREAM_STAGES.map((stage: keyof ILevelStreamStages) => (
            <EditorPanelProperty key={stage} label={stage} value={formatDuration(stream.mean?.[stage] ?? 0)} />
          ))}
          <EditorPanelProperty label={"Mean"} value={formatDuration(stream.mean.total)} />
          <EditorPanelProperty
            label={"Worst"}
            value={
              stream.worst
                ? `${formatDuration(stream.worst.total)} · sector ${stream.worst.sector}`
                : "nothing read yet"
            }
          />
          <EditorPanelProperty label={"Sectors read"} value={stream.sectors} />
        </EditorPanelSection>
      ) : null}

      {stream.planning.reports ? (
        <EditorPanelSection title={"Answering the camera"}>
          <EditorPanelProperty label={"Mean"} value={formatDuration(stream.planning.mean)} />
          <EditorPanelProperty label={"Reports"} value={stream.planning.reports} />
        </EditorPanelSection>
      ) : null}

      <EditorPanelSection title={"Frame"}>
        <EditorPanelProperty label={"Frame time"} value={`${stats.frameTime.toFixed(1)} ms`} />
        <EditorPanelProperty label={"Worst frame"} value={`${stats.worstFrameTime.toFixed(1)} ms`} />
        <EditorPanelProperty
          label={"Drawing"}
          value={`${stats.drawTime.toFixed(1)} ms · worst ${stats.worstDrawTime.toFixed(1)} ms`}
        />
        <EditorPanelProperty label={"Frames a second"} value={stats.framesPerSecond.toFixed(0)} />
        <EditorPanelProperty label={"Taking a sector in"} value={formatDuration(stats.sceneTime)} />
        <EditorPanelProperty label={"Draw calls"} value={stats.draws} />
        <EditorPanelProperty label={"Triangles"} value={stats.triangles} />
      </EditorPanelSection>

      <EditorPanelSection title={"Textures"}>
        <EditorPanelProperty label={"Uploaded"} value={textures.uploaded} />
        <EditorPanelProperty label={"Unusable"} value={problems.length} />
      </EditorPanelSection>

      <EditorPanelSection title={"Budget"}>
        <EditorPanelProperty
          label={"Memory"}
          value={`${formatBytes(stats.bytes)} of ${formatBytes(loadService.residency.memoryBudget)}`}
        />
        <EditorPanelProperty label={"Sector cap"} value={loadService.residency.maxSectors} />
        <EditorPanelProperty label={"Reads at once"} value={loadService.residency.concurrency} />
        <EditorPanelProperty label={"Fills the level in"} value={loadService.residency.isPreloaded ? "yes" : "no"} />
        <EditorPanelProperty label={"Load distance"} value={`${loadService.residency.loadDistance.toFixed(0)} m`} />
        <EditorPanelProperty label={"Keep distance"} value={`${loadService.residency.keepDistance.toFixed(0)} m`} />
      </EditorPanelSection>
    </EditorPanel>
  );
}
