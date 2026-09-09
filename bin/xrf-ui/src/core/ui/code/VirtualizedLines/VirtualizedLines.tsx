import { Box, useTheme } from "@mui/material";
import { LayoutList, useVirtualizer } from "@mui/x-virtualizer";
import { KeyboardEvent, ReactElement, useCallback, useEffect, useId, useMemo, useRef } from "react";

import { getSyntaxColors } from "@/core/syntax/components/syntax.styles";
import { ESyntaxToken } from "@/core/syntax/lib";
import { mergeSx } from "@/core/theme/merge-sx";
import { CODE, MONOSPACE_CHARACTER_WIDTH } from "@/core/theme/tokens";
import { ICodeLine } from "@/core/ui/code/code-line";
import { VirtualizedLinesRow } from "@/core/ui/code/VirtualizedLines/VirtualizedLinesRow";
import { StyledComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IVirtualizedLinesProps extends StyledComponentProps {
  lines: ReadonlyArray<ICodeLine>;
  ariaLabel: string;
  /**
   * Line drawn as selected, by the number it displays.
   */
  selectedLine?: Nullable<number>;
  /**
   * Line to bring into view without selecting it.
   */
  scrollToLine?: Nullable<number>;
  /** A line was chosen, by click or by arrow key. */
  onSelectLine?: (line: ICodeLine) => void;
}

/**
 * Source lines with a gutter, rendering only the window on screen.
 *
 * The listing owns no text. It is addressed by the number a line displays rather than by its position,
 * because that number is what a finding, a section index or a cross-file jump names.
 */
export function VirtualizedLines({
  "data-testid": dataTestId = "virtualized-lines",
  id,
  className,
  sx,
  lines,
  ariaLabel,
  selectedLine = null,
  scrollToLine = null,
  onSelectLine,
}: IVirtualizedLinesProps): ReactElement {
  const theme = useTheme();
  const listId: string = useId();

  const scrollerRef = useRef<HTMLElement | null>(null);
  const layoutRef = useRef<Nullable<LayoutList>>(null);

  if (!layoutRef.current) {
    // The constructor takes these, but `LayoutList.use` reads the refs it is handed in `layoutParams`
    // instead, so nothing ever writes to them. The scroller node is captured below rather than here.
    layoutRef.current = new LayoutList({ container: { current: null }, scroller: { current: null } });
  }

  const colors: Record<ESyntaxToken, string> = useMemo(() => getSyntaxColors(theme), [theme]);

  // Rows are identified by position rather than by the number they show: a resolved document is
  // assembled out of sections, and nothing stops two of its lines from carrying the same number.
  const virtualizerRows = useMemo(
    () => lines.map((line: ICodeLine, index: number) => ({ id: index, model: line })),
    [lines]
  );

  const range = useMemo(() => ({ firstRowIndex: 0, lastRowIndex: lines.length }), [lines.length]);

  const rowIdOf = useCallback((index: number) => `${listId}-line-${index}`, [listId]);

  /** Where a displayed number sits, since every address into this listing is one. */
  const indexOfNumber: ReadonlyMap<number, number> = useMemo(
    () => new Map(lines.map((line: ICodeLine, index: number) => [line.number, index])),
    [lines]
  );

  /**
   * Width of the gutter column, from the widest number it has to hold.
   *
   * The widest is searched for rather than taken from the last line, since a listing is not obliged to
   * number its lines in order.
   */
  const gutterWidth: number = useMemo(() => {
    let widest: number = 1;

    for (const line of lines) {
      if (line.number > widest) {
        widest = line.number;
      }
    }

    const digits: number = Math.max(CODE.minimumGutterDigits, String(widest).length);

    return Math.ceil(digits * MONOSPACE_CHARACTER_WIDTH) + CODE.markIconSize + CODE.gutterGap + CODE.gutterPaddingX * 2;
  }, [lines]);

  // A selection naming a line this document does not have simply draws nothing, and the first arrow
  // key lands on the first line.
  const selectedIndex: number = selectedLine === null ? -1 : (indexOfNumber.get(selectedLine) ?? -1);

  const select = useCallback((line: ICodeLine) => onSelectLine?.(line), [onSelectLine]);

  const virtualizer = useVirtualizer({
    layout: layoutRef.current,
    dimensions: { rowHeight: CODE.lineHeight },
    virtualization: {},
    rows: virtualizerRows,
    range,
    rowCount: lines.length,
    renderRow: (params) => {
      const line: ICodeLine = params.model as unknown as ICodeLine;

      return (
        <VirtualizedLinesRow
          key={params.rowIndex}
          line={line}
          rowId={rowIdOf(params.rowIndex)}
          isSelected={params.rowIndex === selectedIndex}
          gutterWidth={gutterWidth}
          colors={colors}
          onSelect={select}
        />
      );
    },
  });

  const containerProps = virtualizer.store.use(LayoutList.selectors.containerProps);
  const contentProps = virtualizer.store.use(LayoutList.selectors.contentProps);
  const positionerProps = virtualizer.store.use(LayoutList.selectors.positionerProps);

  /**
   * Hands the scroller node to the virtualizer and keeps a reference to it.
   *
   * The virtualizer's own ref is what attaches its scroll and resize listeners, so it has to be called;
   * keeping the node as well is what lets a line be scrolled into view.
   */
  const setScroller = useCallback(
    (node: Nullable<HTMLElement>): void => {
      scrollerRef.current = node;

      const attach: unknown = containerProps.ref;

      if (typeof attach === "function") {
        (attach as (element: Nullable<HTMLElement>) => void)(node);
      } else if (attach) {
        (attach as { current: Nullable<HTMLElement> }).current = node;
      }
    },
    [containerProps.ref]
  );

  /** Scrolls a line into view by arithmetic, since every line is exactly one `CODE.lineHeight` tall. */
  const revealLine = useCallback((index: number): void => {
    const scroller: Nullable<HTMLElement> = scrollerRef.current;

    if (!scroller || index < 0) {
      return;
    }

    const top: number = index * CODE.lineHeight;
    const bottom: number = top + CODE.lineHeight;

    if (top < scroller.scrollTop) {
      scroller.scrollTop = top;
    } else if (bottom > scroller.scrollTop + scroller.clientHeight) {
      scroller.scrollTop = bottom - scroller.clientHeight;
    }
  }, []);

  const moveTo = useCallback(
    (index: number): void => {
      const next: number = Math.min(Math.max(index, 0), lines.length - 1);
      const line: Nullable<ICodeLine> = lines[next] ?? null;

      if (line) {
        select(line);
        revealLine(next);
      }
    },
    [lines, revealLine, select]
  );

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>): void => {
      switch (event.key) {
        case "ArrowDown":
          event.preventDefault();

          return moveTo(selectedIndex + 1);

        case "ArrowUp":
          event.preventDefault();

          return moveTo(selectedIndex - 1);

        case "Home":
          event.preventDefault();

          return moveTo(0);

        case "End":
          event.preventDefault();

          return moveTo(lines.length - 1);

        default:
          return;
      }
    },
    [lines.length, moveTo, selectedIndex]
  );

  useEffect(() => revealLine(selectedIndex), [revealLine, selectedIndex]);

  useEffect(() => {
    if (scrollToLine !== null) {
      revealLine(indexOfNumber.get(scrollToLine) ?? -1);
    }
  }, [indexOfNumber, revealLine, scrollToLine]);

  return (
    <Box
      {...containerProps}
      ref={setScroller}
      // `LayoutList` pins `overflowX` to hidden for a listing with no columns declared, which is right
      // for a tree of ellipsized labels and wrong here: a config line is read to its end or not at all.
      style={{ ...containerProps.style, overflowX: "auto" }}
      aria-activedescendant={selectedIndex === -1 ? undefined : rowIdOf(selectedIndex)}
      aria-label={ariaLabel}
      data-testid={dataTestId}
      id={id}
      className={className}
      role={"listbox"}
      sx={mergeSx(
        {
          backgroundColor: "background.default",
          // A definite height, or the listing grows to its content instead of windowing it.
          height: "100%",
          outline: "none",
          overflow: "auto",
        },
        sx
      )}
      tabIndex={0}
      onKeyDown={onKeyDown}
    >
      <div {...contentProps} />
      <div {...positionerProps} role={"presentation"} />

      {virtualizer.api.getters.getRows()}
    </Box>
  );
}
