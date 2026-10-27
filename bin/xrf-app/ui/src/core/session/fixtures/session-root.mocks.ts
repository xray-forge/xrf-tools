import { runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";

import { SessionRootResource } from "@/core/session/lib/session-root-resource";

/**
 * Puts a root on a resource as a restore would, without the restore's read, for a test whose setup cannot wait for one:
 * held, owned, so a close releases it, and restored.
 *
 * @param resource - Resource to hold the root.
 * @param snapshot - What the backend holds.
 * @returns The same snapshot, so it can be passed on.
 */
export function mockHeldSessionRoot<T>(
  resource: SessionRootResource<T>,
  snapshot: SessionSnapshot<T>
): SessionSnapshot<T> {
  resource["session"].adopt(snapshot);

  runInAction(() => {
    resource["root"] = AsyncState.ready(snapshot);
    resource.isRestored = true;
  });

  return snapshot;
}

/**
 * Lets a resource's root go as a close would have, without the close, for a test that only needs nothing held.
 *
 * @param resource - Resource to hold nothing.
 */
export function mockLetGoSessionRoot<T>(resource: SessionRootResource<T>): void {
  runInAction(() => {
    resource["root"] = AsyncState.idle();
  });
}
