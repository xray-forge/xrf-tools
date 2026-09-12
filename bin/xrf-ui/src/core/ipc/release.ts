import { isTauri } from "@tauri-apps/api/core";

import { Logger } from "@/lib/logging";

/**
 * Tell the backend to drop whatever the editor had open, on the way out.
 *
 * Deactivation cannot await, so the release is started here and its failure is reported rather than propagated: there
 * is no longer a surface to report it to, and the session it names is released again by the next teardown.
 *
 * @param release - Release to start, addressed by whatever the caller still owns.
 */
export function releaseEditorProject(release: () => Promise<unknown>): void {
  if (!isTauri()) {
    return;
  }

  release().catch((error: unknown) => {
    Logger.error("Failed to release editor project on deactivation:", error);
  });
}
