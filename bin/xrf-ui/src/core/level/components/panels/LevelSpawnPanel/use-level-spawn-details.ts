import { Nullable } from "@xrf/types";
import { useEffect, useState } from "react";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { LevelSpawnObjectDetails } from "@/core/ipc/types/xrf-app";
import { AsyncState } from "@/lib/async-state";

/**
 * What the backend says of one spawned object beyond what the tree holds, read when it is chosen.
 *
 * @param sessionId - The level opening it belongs to, or null for none open.
 * @param index - The object, by its place among the level's spawned objects, or null for none chosen.
 * @returns Its details as they are read; an answer for an object chosen since is dropped.
 */
export function useLevelSpawnDetails(
  sessionId: Nullable<string>,
  index: Nullable<number>
): AsyncState<LevelSpawnObjectDetails> {
  const [details, setDetails] = useState<AsyncState<LevelSpawnObjectDetails>>(() => AsyncState.idle());

  useEffect(() => {
    if (sessionId === null || index === null) {
      setDetails(AsyncState.idle());

      return;
    }

    let isCurrent: boolean = true;

    setDetails(AsyncState.loading());
    levelsCommands
      .describeSpawnObject(sessionId, index)
      .then(({ value }) => isCurrent && setDetails(AsyncState.ready(value)))
      .catch((error: unknown) => isCurrent && setDetails(AsyncState.failed(transformError(error))));

    return () => {
      isCurrent = false;
    };
  }, [sessionId, index]);

  return details;
}
