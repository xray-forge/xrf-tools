import { useInjection } from "@wirestate/react";
import { useEffect } from "react";

import { TConfigsReveal } from "@/core/ltx/lib/reveal";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { Nullable } from "@/lib/types/general";

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
