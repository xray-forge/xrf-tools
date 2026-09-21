import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { LevelSurfaceRow } from "@/core/level/components/panels/LevelSurfacesPanel/LevelSurfaceRow";
import {
  ILevelSurfaceSummary,
  listLevelSurfaces,
  listNamedLevelSurfaces,
} from "@/core/level/lib/surface/level-surface-summary";
import { LevelLoadService } from "@/core/level/services";
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
  const service: LevelLoadService = useInjection(LevelLoadService);

  const surfaces: Maybe<ReadonlyArray<XraySurfaceDescriptor>> = service.level.value?.selected.value.surfaces;

  const { named, total } = useMemo(() => {
    const summaries: Array<ILevelSurfaceSummary> = listLevelSurfaces(surfaces ?? []);

    return { named: listNamedLevelSurfaces(summaries), total: summaries.length };
  }, [surfaces]);

  if (!service.level.value) {
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
        <LevelSurfaceRow key={summary.shaderId} summary={summary} />
      ))}
    </EditorPanel>
  );
}
