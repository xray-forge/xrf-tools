import { Typography } from "@mui/material";
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
    <div data-testid={dataTestId} className={"flex h-full min-h-0 flex-col"}>
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
        <div className={"min-h-0 grow"}>
          <VirtualizedTree<TConfigsIndexRow>
            ariaLabel={isList ? "Entries" : "Sections"}
            className={"h-full"}
            items={items}
            expandedIds={NOTHING_EXPANDED}
            selectedId={null}
            onToggleExpanded={noop}
            onSelect={onSelect}
            onActivate={onSelect}
          />
        </div>
      ) : (
        <div className={"p-4 text-center"}>
          <Typography className={"text-text-secondary"} variant={"body2"}>
            {filter ? `No ${isList ? "entry" : "section"} matches that.` : "This document declares no sections."}
          </Typography>
        </div>
      )}
    </div>
  );
}
