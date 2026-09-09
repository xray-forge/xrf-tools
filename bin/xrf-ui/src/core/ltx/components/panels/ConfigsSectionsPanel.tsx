import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { LtxResolvedIndexEntry } from "@/core/bindings/types/xrf-ltx-inspect";
import { ConfigsDocumentService, EConfigsDocumentMode } from "@/core/ltx/services/document";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** No row of this list has children, so nothing is ever expanded and nothing can toggle. */
const NOTHING_EXPANDED: ReadonlySet<string> = new Set();

/**
 * Every section the open document holds, as a jump list.
 *
 * Flat rather than a tree, because sections have no hierarchy - but virtualized all the same, since a resolved
 * `system.ltx` indexes 23,500 of them.
 */
export function ConfigsSectionsPanel({
  "data-testid": dataTestId = "configs-sections-panel",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const resolvedService: ConfigsResolvedService = useInjection(ConfigsResolvedService);

  const [filter, setFilter] = useState<string>("");

  const isResolved: boolean = documentService.mode === EConfigsDocumentMode.RESOLVED;

  // Whichever view is on screen names its own sections: the resolved index, or the headers the file declares.
  const items: Array<ITreeNode<string>> = useMemo(() => {
    const named: Array<string> = isResolved
      ? resolvedService.visibleSections.map((section: LtxResolvedIndexEntry) => section.name)
      : (documentService.document.value?.structure.sections ?? []).map((section) => section.name);

    const query: string = filter.trim().toLowerCase();
    const matched: Array<string> = query
      ? named.filter((name: string) => name.toLowerCase().includes(query))
      : named;

    return matched.map((name: string) => ({ id: `section:${name}`, label: name }));
  }, [documentService.document.value, filter, isResolved, resolvedService.visibleSections]);

  const onSelect = useCallback(
    (item: ITreeNode<string>) => documentService.revealSection(item.label),
    [documentService]
  );

  return (
    <Box data-testid={dataTestId} sx={{ display: "flex", flexDirection: "column", minHeight: 0, height: "100%" }}>
      <EditorFilterInput
        query={filter}
        placeholder={"Filter sections"}
        ariaLabel={"Filter sections"}
        onQueryChange={setFilter}
      />

      {items.length ? (
        <Box sx={{ flexGrow: 1, minHeight: 0 }}>
          <VirtualizedTree<string>
            ariaLabel={"Sections"}
            items={items}
            expandedIds={NOTHING_EXPANDED}
            selectedId={null}
            sx={{ height: "100%" }}
            onToggleExpanded={() => undefined}
            onSelect={onSelect}
            onActivate={onSelect}
          />
        </Box>
      ) : (
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            {filter ? "No section matches that." : "This document declares no sections."}
          </Typography>
        </Box>
      )}
    </Box>
  );
}
