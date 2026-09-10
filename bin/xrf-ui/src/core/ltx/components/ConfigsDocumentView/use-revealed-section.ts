import { useInjection } from "@wirestate/react";
import { useEffect } from "react";

import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { Nullable } from "@/lib/types/general";

/**
 * The section a panel asked to be brought into view, consumed by reading it.
 *
 * A reveal is an event wearing the clothes of state: the Sections panel names a section, whichever view is open
 * scrolls to it, and the request is spent. Clearing it is what lets the same section be asked for twice - a second
 * click would otherwise change no prop and scroll nowhere - and it belongs to the component that renders the listing,
 * because React runs the listing's own effects first and it has already scrolled by the time this one runs.
 *
 * @returns The section to reveal on this render, or null when nothing was asked for.
 */
export function useRevealedSection(): Nullable<string> {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const revealed: Nullable<string> = documentService.revealedSection;

  useEffect(() => {
    if (revealed) {
      documentService.clearRevealed();
    }
  }, [documentService, revealed]);

  return revealed;
}
