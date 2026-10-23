import { useInjection } from "@wirestate/react";
import { useEffect } from "react";

import { SessionChangesService } from "./session-changes.service";

/**
 * Subscribes the window to committed changes for as long as the shell stands; the subscription is never closed, as
 * the backend drops it with the page.
 */
export function useSessionChanges(): void {
  const changes: SessionChangesService = useInjection(SessionChangesService);

  useEffect(() => {
    void changes.listen();
  }, [changes]);
}
