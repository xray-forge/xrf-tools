import { useInjection } from "@wirestate/react";
import { RefObject, useEffect } from "react";

import { LauncherSearchService } from "@/core/launcher/services/launcher-search";
import { Nullable } from "@/lib/types/general";

/**
 * Connects a search field to the focus command.
 *
 * @param inputRef - Field the command focuses and selects.
 */
export function useLauncherSearchFocus(inputRef: RefObject<Nullable<HTMLInputElement>>): void {
  const launcherSearchService: LauncherSearchService = useInjection(LauncherSearchService);

  // Read during render so the calling component tracks it; the effect below acts on each new request.
  const revision: number = launcherSearchService.focusRevision;

  useEffect(() => {
    launcherSearchService.setMounted(true);

    return () => launcherSearchService.setMounted(false);
  }, [launcherSearchService]);

  useEffect(() => {
    if (revision === 0) {
      return;
    }

    inputRef.current?.focus();
    inputRef.current?.select();
  }, [inputRef, revision]);
}
