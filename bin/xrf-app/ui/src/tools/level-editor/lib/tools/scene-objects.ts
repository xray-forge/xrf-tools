import { SceneObjectDescription, SceneObjectId, SceneObjectToolDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

/**
 * @param tools - Every object tool's objects, as a scene's listing holds them.
 * @param id - A scene object, by its tool and index.
 * @returns The object as the listing describes it; null where the listing holds it not.
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
