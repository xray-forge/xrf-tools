import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useMemo, useState } from "react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { LevelSurfaceRow } from "@/core/level/components/panels/LevelSurfacesPanel/LevelSurfaceRow";
import {
  ILevelSurfaceDressing,
  ILevelTextureReport,
  listLevelSurfaceDressing,
} from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelSurfaceGeometry, NO_LEVEL_SURFACE_GEOMETRY } from "@/core/level/lib/surface/level-surface-geometry";
import {
  ILevelSurfaceSummary,
  listLevelSurfaces,
  listNamedLevelSurfaces,
} from "@/core/level/lib/surface/level-surface-summary";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Maybe } from "@/lib/types/general";

/**
 * How every surface of the open level is drawn, one row per entry of its shader table.
 */
export function LevelSurfacesPanel({
  "data-testid": dataTestId = "level-surfaces-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const held: ReadonlyArray<number> = loadService.sectorReport.held;
  const report: ILevelTextureReport = loadService.textureReport;
  const surfaces: Maybe<ReadonlyArray<XraySurfaceDescriptor>> = loadService.level.value?.selected.value.surfaces;

  // Asked for rather than read: measuring it samples the coordinates of every draw of every sector held, and
  // the viewport that holds them may not be on this thread.
  const [drawn, setDrawn] = useState<ReadonlyMap<number, ILevelSurfaceGeometry>>(new Map());

  const { named, total } = useMemo(() => {
    const summaries: Array<ILevelSurfaceSummary> = listLevelSurfaces(surfaces ?? []);

    return { named: listNamedLevelSurfaces(summaries), total: summaries.length };
  }, [surfaces]);

  const dressed: ReadonlyMap<number, Array<ILevelSurfaceDressing>> = useMemo(
    () =>
      new Map(
        named.map((summary: ILevelSurfaceSummary) => [
          summary.shaderId,
          listLevelSurfaceDressing(summary.textures, report),
        ])
      ),
    [named, report]
  );

  useEffect(() => {
    let isCurrent: boolean = true;

    void viewportService.measureSurfaceGeometry().then((measured) => {
      if (isCurrent) {
        setDrawn(measured);
      }
    });

    return () => {
      isCurrent = false;
    };
  }, [viewportService, held]);

  if (!loadService.level.value) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Surfaces"}>
        <EditorPanelEmpty label={"No level open. Open one to see how its surfaces are drawn."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Surfaces"}>
      <EditorPanelSection title={"Table"} isFirst>
        <EditorPanelProperty label={"Entries"} value={total} />
        <EditorPanelProperty label={"Named"} value={named.length} />
      </EditorPanelSection>

      {named.map((summary: ILevelSurfaceSummary) => (
        <LevelSurfaceRow
          key={summary.shaderId}
          summary={summary}
          dressing={dressed.get(summary.shaderId) ?? []}
          geometry={drawn.get(summary.shaderId) ?? NO_LEVEL_SURFACE_GEOMETRY}
        />
      ))}
    </EditorPanel>
  );
}
