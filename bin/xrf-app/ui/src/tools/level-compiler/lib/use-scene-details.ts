import { Nullable } from "@xrf/types";
import { useEffect, useState } from "react";

import { isPathPresent } from "@/core/path/presence";
import { getSceneDetailsPart } from "@/tools/level-compiler/lib/scene-details";

/** Whether one scene holds details, kept with the scene it answers for. */
interface ISceneDetailsAnswer {
  scenePath: string;
  hasDetails: boolean;
}

/**
 * Whether the scene holds details, by its detail part ({@link getSceneDetailsPart}).
 *
 * @param scenePath - The scene's `.level`, or null for none named.
 * @returns Whether it holds them, or null while unknown: no scene named, or the look still answering.
 */
export function useSceneDetails(scenePath: Nullable<string>): Nullable<boolean> {
  const [answer, setAnswer] = useState<Nullable<ISceneDetailsAnswer>>(null);

  useEffect(() => {
    if (!scenePath) {
      return;
    }

    let isCurrent: boolean = true;

    void isPathPresent(getSceneDetailsPart(scenePath)).then((hasDetails: boolean) => {
      if (isCurrent) {
        setAnswer({ scenePath, hasDetails });
      }
    });

    return () => {
      isCurrent = false;
    };
  }, [scenePath]);

  return answer !== null && answer.scenePath === scenePath ? answer.hasDetails : null;
}
