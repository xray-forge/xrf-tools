import { Alert, Box, Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { LtxResolvedIndex } from "@/core/bindings/types/xrf-ltx-inspect";
import { IResolvedDocument, toResolvedDocument } from "@/core/ltx/lib/resolved";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { ICodeLineRange } from "@/core/ui/code/code-line";
import { VirtualizedLines } from "@/core/ui/code/VirtualizedLines";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * What the open config's entry point resolves to, with every value's origin beside it.
 *
 * The index gives the document its height before a single body arrives, and bodies are asked for as they scroll into
 * view. A vanilla `system.ltx` resolves to 23,500 sections holding 293,000 fields, so fetching it whole is not an
 * option and neither is guessing how tall it is.
 */
export function ConfigsResolvedView({
  "data-testid": dataTestId = "configs-resolved-view",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const resolvedService: ConfigsResolvedService = useInjection(ConfigsResolvedService);

  const index: Nullable<LtxResolvedIndex> = resolvedService.index.value;
  const narrowedTo: Nullable<string> = resolvedService.narrowedTo;
  const revealed: Nullable<string> = documentService.revealedSection;

  // Rebuilt when a page lands, which `revision` is the signal for: the map itself is mutated in place, because copying
  // one that grows to 23,500 entries per page would cost more than every fetch put together.
  const revision: number = resolvedService.revision;
  const visibleSections = resolvedService.visibleSections;

  const document: Nullable<IResolvedDocument> = useMemo(() => {
    // Counting the pages that have landed is what rebuilds the layout as bodies arrive: they are written into a map
    // that is mutated in place, because copying one that grows to 23,500 entries per page would cost more than every
    // fetch put together.
    void revision;

    // Narrowing is the caller's, so the builder lays out exactly the sections it is handed and never re-orders them.
    return index ? toResolvedDocument({ ...index, sections: visibleSections }, resolvedService.sections) : null;
  }, [index, revision, resolvedService.sections, visibleSections]);

  const onVisibleRangeChange = useCallback(
    (range: ICodeLineRange) => {
      if (document) {
        void resolvedService.request(document.getSectionsInRange(range.firstLine, range.lastLine));
      }
    },
    [document, resolvedService]
  );

  // Cleared once the listing below has acted on it, which child effects run before this one does. Without the clear a
  // second click on the same section would change no prop and scroll nowhere.
  useEffect(() => {
    if (revealed) {
      documentService.clearRevealed();
    }
  }, [documentService, revealed]);

  if (resolvedService.index.isFailed) {
    return (
      <ErrorState
        data-testid={dataTestId}
        title={"Cannot resolve this config"}
        description={resolvedService.index.error?.message ?? "The resolution failed."}
        onRetry={() => void (resolvedService.entry && resolvedService.open(resolvedService.entry))}
      />
    );
  }

  if (!document) {
    return <DelayedProgress data-testid={dataTestId} />;
  }

  return (
    <Box
      data-testid={dataTestId}
      sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}
    >
      {narrowedTo ? (
        <Alert
          severity={"info"}
          variant={"outlined"}
          square
          action={
            <Button size={"small"} onClick={() => resolvedService.narrowTo(null)}>
              Show all
            </Button>
          }
        >
          {`Showing the ${visibleSections.length} section(s) this config declares, resolved through ` +
            `${resolvedService.entry ?? "its entry point"}.`}
        </Alert>
      ) : null}

      <VirtualizedLines
        data-testid={"configs-resolved-lines"}
        ariaLabel={`Resolved sections of ${resolvedService.entry ?? "this config"}`}
        lines={document.lines}
        scrollToLine={revealed ? document.getSectionLine(revealed) : null}
        sx={{ flexGrow: 1, minWidth: 0, minHeight: 0 }}
        onVisibleRangeChange={onVisibleRangeChange}
      />
    </Box>
  );
}
