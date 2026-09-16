import { Box, Typography } from "@mui/material";
import { SxProps, Theme } from "@mui/material/styles";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { LtxFileStructure } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsDocumentService, EConfigsDocumentMode } from "@/core/ltx/services/document";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { noop } from "@/lib/callbacks/noop";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import {
  filterConfigsIndex,
  isListConfig,
  TConfigsIndexRow,
  toAuthoredIndex,
  toResolvedIndex,
} from "./configs-index-rows";

const NOTHING_EXPANDED: ReadonlySet<string> = new Set();
const TREE_SX: SxProps<Theme> = { height: "100%" };

/**
 * Everything the open document declares, as a jump list.
 */
export function ConfigsSectionsPanel({
  "data-testid": dataTestId = "configs-sections-panel",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const resolvedService: ConfigsResolvedService = useInjection(ConfigsResolvedService);

  const [filter, setFilter] = useState<string>("");

  const isResolved: boolean = documentService.mode === EConfigsDocumentMode.RESOLVED;
  const structure: Nullable<LtxFileStructure> = documentService.document.value?.structure ?? null;

  // What the rows stand for, which is what the heading and the empty line have to call them.
  const isList: boolean = !isResolved && isListConfig(structure);

  // Whichever view is on screen names its own rows: the resolved index, or the file as the parser read it.
  const items: Array<ITreeNode<TConfigsIndexRow>> = useMemo(
    () =>
      filterConfigsIndex(
        isResolved ? toResolvedIndex(resolvedService.visibleSections) : toAuthoredIndex(structure),
        filter
      ),
    [filter, isResolved, resolvedService.visibleSections, structure]
  );

  const onSelect = useCallback(
    (item: ITreeNode<TConfigsIndexRow>) => {
      const row: Nullable<TConfigsIndexRow> = item.payload ?? null;

      if (!row) {
        return;
      }

      // A section is revealed by name, which also makes it the selection the Scheme panel explains. An entry belongs to
      // no section, so it is opened at its line and selects nothing.
      if (row.kind === "section") {
        documentService.revealSection(row.name);
      } else if (documentService.selected) {
        void documentService.openAt(documentService.selected, row.line);
      }
    },
    [documentService]
  );

  return (
    <Box data-testid={dataTestId} sx={{ display: "flex", flexDirection: "column", minHeight: 0, height: "100%" }}>
      <EditorSearchHeader
        title={isList ? "Entries" : "Sections"}
        count={items.length}
        query={filter}
        placeholder={isList ? "Filter entries" : "Filter sections"}
        ariaLabel={isList ? "Filter entries" : "Filter sections"}
        onClear={() => setFilter("")}
        onQueryChange={setFilter}
      />

      {items.length ? (
        <Box sx={{ flexGrow: 1, minHeight: 0 }}>
          <VirtualizedTree<TConfigsIndexRow>
            ariaLabel={isList ? "Entries" : "Sections"}
            items={items}
            expandedIds={NOTHING_EXPANDED}
            selectedId={null}
            sx={TREE_SX}
            onToggleExpanded={noop}
            onSelect={onSelect}
            onActivate={onSelect}
          />
        </Box>
      ) : (
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            {filter ? `No ${isList ? "entry" : "section"} matches that.` : "This document declares no sections."}
          </Typography>
        </Box>
      )}
    </Box>
  );
}
