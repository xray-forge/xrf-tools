import { Box } from "@mui/material";
import { LayoutList, useVirtualizer } from "@mui/x-virtualizer";
import { ReactElement, useCallback, useId, useMemo, useRef } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { ARCHIVE_OVERRIDE_ROW_HEIGHT, EArchiveOverrideRow, TArchiveOverrideRow } from "./archive-override-rows";
import { ArchiveOverrideCopyRow } from "./ArchiveOverrideCopyRow";
import { ArchiveOverridePathRow } from "./ArchiveOverridePathRow";

export interface IArchiveOverrideListProps extends BaseComponentProps {
  ariaLabel: string;
  rows: ReadonlyArray<TArchiveOverrideRow>;
  /** Opens the file a path row names. */
  onOpen: (name: string) => void;
}

/**
 * The contested paths and their copies, rendering only the rows on screen.
 */
export function ArchiveOverrideList({
  "data-testid": dataTestId = "archive-override-list",
  id,
  className,
  rows,
  ariaLabel,
  onOpen,
}: IArchiveOverrideListProps): ReactElement {
  const listId: string = useId();
  const layoutRef = useRef<Nullable<LayoutList>>(null);

  if (!layoutRef.current) {
    layoutRef.current = new LayoutList({ container: { current: null }, scroller: { current: null } });
  }

  const virtualizerRows = useMemo(() => rows.map((row: TArchiveOverrideRow) => ({ id: row.id, model: row })), [rows]);

  const range = useMemo(() => ({ firstRowIndex: 0, lastRowIndex: rows.length }), [rows.length]);

  const virtualizer = useVirtualizer({
    layout: layoutRef.current,
    dimensions: { rowHeight: ARCHIVE_OVERRIDE_ROW_HEIGHT },
    virtualization: {},
    rows: virtualizerRows,
    range,
    rowCount: rows.length,
    renderRow: (params) => {
      const row: TArchiveOverrideRow = params.model as unknown as TArchiveOverrideRow;

      return row.kind === EArchiveOverrideRow.PATH ? (
        <ArchiveOverridePathRow key={row.id} id={`${listId}-row-${params.rowIndex}`} row={row} onOpen={onOpen} />
      ) : (
        <ArchiveOverrideCopyRow key={row.id} id={`${listId}-row-${params.rowIndex}`} row={row} />
      );
    },
  });

  const containerProps = virtualizer.store.use(LayoutList.selectors.containerProps);
  const contentProps = virtualizer.store.use(LayoutList.selectors.contentProps);
  const positionerProps = virtualizer.store.use(LayoutList.selectors.positionerProps);

  const setScroller = useCallback(
    (node: Nullable<HTMLElement>): void => {
      const attach: unknown = containerProps.ref;

      if (typeof attach === "function") {
        (attach as (element: Nullable<HTMLElement>) => void)(node);
      } else if (attach) {
        (attach as { current: Nullable<HTMLElement> }).current = node;
      }
    },
    [containerProps.ref]
  );

  return (
    <Box
      {...containerProps}
      ref={setScroller}
      data-testid={dataTestId}
      id={id}
      className={className}
      role={"list"}
      aria-label={ariaLabel}
      sx={{ height: "100%", overflow: "auto", outline: "none" }}
    >
      <div {...contentProps} />
      <div {...positionerProps} role={"presentation"} />

      {virtualizer.api.getters.getRows()}
    </Box>
  );
}
