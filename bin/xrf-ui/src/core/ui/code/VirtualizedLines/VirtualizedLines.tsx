import { Box, useTheme } from "@mui/material";
import { LayoutList, RenderContext, useVirtualizer, Virtualization } from "@mui/x-virtualizer";
import { KeyboardEvent, ReactElement, useCallback, useEffect, useId, useMemo, useRef } from "react";

import { getSyntaxColors } from "@/core/syntax/components/syntax.styles";
import { ESyntaxToken } from "@/core/syntax/lib";
import { mergeSx } from "@/core/theme/merge-sx";
import { CODE, MONOSPACE_CHARACTER_WIDTH } from "@/core/theme/tokens";
import { ICodeLine, ICodeLineRange } from "@/core/ui/code/code-line";
import { VirtualizedLinesRow } from "@/core/ui/code/VirtualizedLines/VirtualizedLinesRow";
import { StyledComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * Where a reveal leaves the line it was handed.
 *
 * A step of the selection moves the listing as little as it can, because the line beside the one just left has to
 * stay where the eye already is. An address into the document is the opposite: a section is read downwards from its
 * header, and one revealed against the bottom edge shows its name with none of its body.
 */
const enum ECodeLineReveal {
  NEAREST,
  START,
}

interface IVirtualizedLinesProps extends StyledComponentProps {
  lines: ReadonlyArray<ICodeLine>;
  ariaLabel: string;
  /**
   * Line drawn as selected, by the number it displays.
   */
  selectedLine?: Nullable<number>;
  /**
   * Line to bring into view without selecting it, placed at the top of the listing.
   */
  scrollToLine?: Nullable<number>;
  /** A line was chosen, by click or by arrow key. */
  onSelectLine?: (line: ICodeLine) => void;
  /**
   * The stretch of lines on screen changed, reported by the numbers they display.
   *
   * Fired for the window as it is rendered, buffer included, and only when that window moves - a listing whose
   * content is fetched as it scrolls would otherwise ask for the same page on every scroll frame.
   */
  onVisibleRangeChange?: (range: ICodeLineRange) => void;
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
  onVisibleRangeChange,
}: IVirtualizedLinesProps): ReactElement {
  const theme = useTheme();
  const listId: string = useId();

  const scrollerRef = useRef<HTMLElement | null>(null);
  const reportedRangeRef = useRef<Nullable<ICodeLineRange>>(null);
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

  /**
   * The rows the virtualizer decided to render, as it decided them.
   */
  const renderContext: RenderContext = virtualizer.store.use(Virtualization.selectors.renderContext);

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
  const revealLine = useCallback((index: number, reveal: ECodeLineReveal): void => {
    const scroller: Nullable<HTMLElement> = scrollerRef.current;

    if (!scroller || index < 0) {
      return;
    }

    const top: number = index * CODE.lineHeight;

    // Asked for unconditionally rather than only when the line is off screen: a section revealed is a section about
    // to be read, and the scroller clamps this itself for the last screenful of the document.
    if (reveal === ECodeLineReveal.START) {
      scroller.scrollTop = top;

      return;
    }

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
        revealLine(next, ECodeLineReveal.NEAREST);
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

  useEffect(() => revealLine(selectedIndex, ECodeLineReveal.NEAREST), [revealLine, selectedIndex]);

  useEffect(() => {
    if (scrollToLine !== null) {
      revealLine(indexOfNumber.get(scrollToLine) ?? -1, ECodeLineReveal.START);
    }
  }, [indexOfNumber, revealLine, scrollToLine]);

  useEffect(() => {
    if (!onVisibleRangeChange) {
      return;
    }

    // The interval is clamped the way the rows themselves are: the virtualizer's last position is exclusive and may
    // sit past the end of a document that just got shorter.
    const first: number = Math.max(renderContext.firstRowIndex, 0);
    const last: number = Math.min(renderContext.lastRowIndex, lines.length) - 1;

    if (last < first) {
      return;
    }

    const range: ICodeLineRange = { firstLine: lines[first].number, lastLine: lines[last].number };
    const reported: Nullable<ICodeLineRange> = reportedRangeRef.current;

    // Compared by the numbers reported and not by the positions behind them, because the numbers are the whole answer:
    // a document swapped for one numbered identically is showing the same lines, and a scroll that lands on the same
    // window has nothing new to say.
    if (reported && reported.firstLine === range.firstLine && reported.lastLine === range.lastLine) {
      return;
    }

    reportedRangeRef.current = range;
    onVisibleRangeChange(range);
  }, [lines, onVisibleRangeChange, renderContext]);

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
