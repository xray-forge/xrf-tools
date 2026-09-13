import { Alert, Box, Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { LtxResolvedIndexEntry } from "@/core/ipc/types/xrf-ltx-inspect";
import { useRevealed } from "@/core/ltx/components/ConfigsDocumentView/use-revealed";
import { IResolvedLayout, toResolvedLayout } from "@/core/ltx/lib/resolved";
import { TConfigsReveal } from "@/core/ltx/lib/reveal";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { ICodeLine, ICodeLineRange, ICodeLineSource } from "@/core/ui/code/code-line";
import { VirtualizedLines } from "@/core/ui/code/VirtualizedLines";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * What the open config's entry point resolves to, with every value's origin beside it.
 *
 * The index gives the document its height before a single body arrives, and bodies are asked for as they scroll into
 * view. Anomaly's `system.ltx` resolves to 11,870 sections holding 527,000 fields, so fetching it whole is not an
 * option, neither is guessing how tall it is, and neither is building its 551,000 lines - the listing is handed a
 * source that builds the forty on screen.
 */
export function ConfigsResolvedView({
  "data-testid": dataTestId = "configs-resolved-view",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const resolvedService: ConfigsResolvedService = useInjection(ConfigsResolvedService);

  const narrowedTo: Nullable<string> = resolvedService.narrowedTo;
  const revealed: Nullable<TConfigsReveal> = useRevealed();

  // Read here so a landed page re-renders this view: the map behind the source is mutated in place, because copying
  // one that grows to 11,870 entries per page would cost more than every fetch put together.
  const revision: number = resolvedService.revision;
  const visibleSections: ReadonlyArray<LtxResolvedIndexEntry> = resolvedService.visibleSections;

  // Where the lines sit, which the index alone decides. Not rebuilt when a page lands: a body changes what a line says
  // and never how many there are or where one is, and this document is half a million lines long. Before an index
  // arrives there are no sections, which lays out as the empty document the gate below replaces anyway.
  const layout: IResolvedLayout = useMemo(() => toResolvedLayout(visibleSections), [visibleSections]);

  // A fresh view over the same layout for each page that lands, which is what redraws the forty lines on screen.
  const source: ICodeLineSource = useMemo(() => {
    // Counting the pages that have landed is what says the map behind the source holds more than it did: the map is
    // mutated in place rather than replaced, so nothing else about it changes when a page arrives.
    void revision;

    return layout.toSource(resolvedService.sections);
  }, [layout, resolvedService, revision]);

  const onVisibleRangeChange = useCallback(
    (range: ICodeLineRange) => void resolvedService.request(layout.getSectionsInRange(range.firstLine, range.lastLine)),
    [layout, resolvedService]
  );

  // Which section a click landed in, which is the one the Scheme panel explains. A line of a resolved document belongs
  // to exactly one section, gap included, so the range of one line is the answer.
  const onSelectLine = useCallback(
    (line: ICodeLine) => documentService.selectSection(layout.getSectionsInRange(line.number, line.number)[0] ?? null),
    [documentService, layout]
  );

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

  if (!resolvedService.isReady) {
    return <DelayedProgress data-testid={dataTestId} />;
  }

  const showAll: ReactElement = (
    <Button size={"small"} onClick={() => resolvedService.narrowTo(null)}>
      Show all
    </Button>
  );

  if (!visibleSections.length) {
    return (
      <EmptyState
        data-testid={dataTestId}
        title={narrowedTo ? "This config declares no sections" : "This entry point resolves to nothing"}
        description={
          narrowedTo
            ? `${narrowedTo} contributes no sections of its own to ${resolvedService.entry ?? "its entry point"}. ` +
              "A config that only includes others reads this way, and so does one whose sections a patch deleted."
            : `${resolvedService.entry ?? "This entry point"} holds no sections once resolved.`
        }
        action={narrowedTo ? showAll : undefined}
      />
    );
  }

  return (
    <Box
      data-testid={dataTestId}
      sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}
    >
      {narrowedTo ? (
        <Alert severity={"info"} variant={"outlined"} square action={showAll}>
          {`Showing the ${visibleSections.length} section(s) this config declares, resolved through ` +
            `${resolvedService.entry ?? "its entry point"}.`}
        </Alert>
      ) : null}

      <VirtualizedLines
        data-testid={"configs-resolved-lines"}
        ariaLabel={`Resolved sections of ${resolvedService.entry ?? "this config"}`}
        source={source}
        // A line of a config means nothing here: this document is assembled out of sections and holds no line of any
        // file, which is why a jump from Problems opens the authored view instead.
        scrollToLine={revealed?.kind === "section" ? layout.getSectionLine(revealed.section) : null}
        sx={{ flexGrow: 1, minWidth: 0, minHeight: 0 }}
        onSelectLine={onSelectLine}
        onVisibleRangeChange={onVisibleRangeChange}
      />
    </Box>
  );
}
