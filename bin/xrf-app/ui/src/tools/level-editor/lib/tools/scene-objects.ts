import { SceneObjectDescription, SceneObjectId, SceneObjectToolDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

/**
 * A scene object as the listing describes it, or null where the listing lacks it.
 */
export function findSceneObject(
  tools: ReadonlyArray<SceneObjectToolDescription>,
  id: SceneObjectId
): Nullable<SceneObjectDescription> {
  return (
    tools
      .find((tool: SceneObjectToolDescription) => tool.tool === id.tool)
      ?.objects.find((it: SceneObjectDescription) => it.id.index === id.index) ?? null
  );
}
