import { useInjection } from "@wirestate/react";
import { ELevelObjectId } from "@xrf/ipc/types/xrf-app-levels";
import { XrayRevision } from "@xrf/ipc/types/xrf-vfs";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useEffect } from "react";

import { ELevelPick, TLevelObjectPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelViewerObjectsService } from "@/tools/level-viewer/services/level-viewer-objects";

import { LevelObjectDetails } from "./LevelObjectDetails";

/**
 * What the selection's active level object is, whichever of the viewport and the Outliner chose it, and how many more
 * are selected with it.
 */
export function LevelInspectorPanel({
  "data-testid": dataTestId = "level-inspector-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const objectsService: LevelViewerObjectsService = useInjection(LevelViewerObjectsService);

  const picked: Nullable<TLevelPick> = viewportService.picked;
  const active: Nullable<TLevelObjectPick> = picked?.kind === ELevelPick.OBJECT ? picked : null;
  const spawnIndex: Nullable<number> =
    active?.object.id.source === ELevelObjectId.SPAWN ? active.object.id.index : null;
  const sessionId: Nullable<string> = loadService.level.value?.selected.sessionId ?? null;
  const revision: Nullable<XrayRevision> = loadService.objectsRevision;

  // The object shown is read again for a listing read again, which may say something else of it.
  useEffect(() => {
    void objectsService.describe(spawnIndex);
  }, [objectsService, revision, sessionId, spawnIndex]);

  if (!active) {
    return (
      <EditorPanelEmpty
        data-testid={dataTestId}
        id={id}
        className={className}
        label={"Nothing selected. Pick an object in the viewport or the Outliner."}
      />
    );
  }

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <LevelObjectDetails
        object={active.object}
        visual={active.visual}
        details={objectsService.details.state}
        others={Math.max(viewportService.selected.length - 1, 0)}
        onGoTo={() => objectsService.frameSelection()}
      />
    </div>
  );
}
