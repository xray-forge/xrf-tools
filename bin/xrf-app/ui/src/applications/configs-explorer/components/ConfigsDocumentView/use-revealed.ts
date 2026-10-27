import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { useEffect } from "react";

import { TConfigsReveal } from "@/applications/configs-explorer/lib/reveal";
import { ConfigsDocumentService } from "@/applications/configs-explorer/services/configs-document";

/**
 * What a panel asked to be brought into view, consumed by reading it.
 *
 * @returns What to reveal on this render, or null when nothing was asked for.
 */
export function useRevealed(): Nullable<TConfigsReveal> {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const revealed: Nullable<TConfigsReveal> = documentService.revealed;

  useEffect(() => {
    if (revealed) {
      documentService.clearRevealed();
    }
  }, [documentService, revealed]);

  return revealed;
}
