import { Box, useTheme } from "@mui/material";
import { LayoutList, RenderContext, useVirtualizer, Virtualization } from "@mui/x-virtualizer";
import { KeyboardEvent, ReactElement, useCallback, useEffect, useId, useMemo, useRef } from "react";

import { getSyntaxColors } from "@/core/syntax/components/syntax.styles";
import { ESyntaxToken } from "@/core/syntax/lib";
import { mergeSx } from "@/core/theme/merge-sx";
import { CODE, MONOSPACE_CHARACTER_WIDTH } from "@/core/theme/tokens";
import { ICodeLine, ICodeLineRange, ICodeLineSource } from "@/core/ui/code/code-line";
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

/**
 * What every row entry the virtualizer holds points at.
 *
 * The virtualizer wants an entry per row, and it hands the entry's `id` back to `renderRow` - which is all this
 * listing needs, since the line itself comes from the source. One shared model for every row rather than a line each:
 * a resolved `system.ltx` has 551,000 rows, and the entries exist to be counted and identified, not read.
 */
const ROW_MODEL: Readonly<Record<string, never>> = Object.freeze({});

/** The last window handed to `onVisibleRangeChange`, and the document it was a window into. */
interface IReportedWindow {
  layout: object;
  range: ICodeLineRange;
}

interface IVirtualizedLinesProps extends StyledComponentProps {
  /**
   * The document to draw.
   *
   * A source rather than an array so a document of hundreds of thousands of lines is never held as one - see
   * {@link ICodeLineSource}. Hand over a new source to redraw what is on screen after its content changed.
   */
  source: ICodeLineSource;
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
  source,
  ariaLabel,
  selectedLine = null,
  scrollToLine = null,
  onSelectLine,
  onVisibleRangeChange,
}: IVirtualizedLinesProps): ReactElement {
  const theme = useTheme();
  const listId: string = useId();

  const scrollerRef = useRef<HTMLElement | null>(null);
  const reportedWindowRef = useRef<Nullable<IReportedWindow>>(null);
  const layoutRef = useRef<Nullable<LayoutList>>(null);

  if (!layoutRef.current) {
    // The constructor takes these, but `LayoutList.use` reads the refs it is handed in `layoutParams`
    // instead, so nothing ever writes to them. The scroller node is captured below rather than here.
    layoutRef.current = new LayoutList({ container: { current: null }, scroller: { current: null } });
  }

  const colors: Record<ESyntaxToken, string> = useMemo(() => getSyntaxColors(theme), [theme]);
  const count: number = source.count;

  // Rows are identified by position rather than by the number they show: a resolved document is
  // assembled out of sections, and nothing stops two of its lines from carrying the same number.
  // Kept for as long as the document is that tall, because a page of content landing changes what the
  // lines say and not how many there are - and rebuilding half a million entries per page is the cost
  // this listing exists to avoid.
  const virtualizerRows = useMemo(
    () => Array.from({ length: count }, (_, index: number) => ({ id: index, model: ROW_MODEL })),
    [count]
  );

  const range = useMemo(() => ({ firstRowIndex: 0, lastRowIndex: count }), [count]);

  const rowIdOf = useCallback((index: number) => `${listId}-line-${index}`, [listId]);

  /**
   * Width of the gutter column, from the widest number it has to hold.
   *
   * Asked of the source rather than searched for, since a listing is not obliged to number its lines in
   * order and a source large enough to matter knows its own widest without walking anything.
   */
  const gutterWidth: number = useMemo(() => {
    const digits: number = Math.max(CODE.minimumGutterDigits, String(source.widestNumber).length);

    return Math.ceil(digits * MONOSPACE_CHARACTER_WIDTH) + CODE.markIconSize + CODE.gutterGap + CODE.gutterPaddingX * 2;
  }, [source]);

  // A selection naming a line this document does not have simply draws nothing, and the first arrow
  // key lands on the first line.
  const selectedIndex: number = selectedLine === null ? -1 : source.indexOfLine(selectedLine);

  const select = useCallback((line: ICodeLine) => onSelectLine?.(line), [onSelectLine]);

  const virtualizer = useVirtualizer({
    layout: layoutRef.current,
    dimensions: { rowHeight: CODE.lineHeight },
    virtualization: {},
    rows: virtualizerRows,
    range,
    rowCount: count,
    renderRow: (params) => {
      // The entry's own id, which is the position this listing gave it: `rowIndex` counts from the page
      // the virtualizer was handed, and this listing hands it the whole document as one page.
      const index: number = params.id as number;

      return (
        <VirtualizedLinesRow
          key={index}
          line={source.getLine(index)}
          rowId={rowIdOf(index)}
          isSelected={index === selectedIndex}
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
      const next: number = Math.min(Math.max(index, 0), count - 1);
      const line: Nullable<ICodeLine> = next < 0 ? null : source.getLine(next);

      if (line) {
        select(line);
        revealLine(next, ECodeLineReveal.NEAREST);
      }
    },
    [count, revealLine, select, source]
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

          return moveTo(count - 1);

        default:
          return;
      }
    },
    [count, moveTo, selectedIndex]
  );

  // A different document starts at its beginning
  useEffect(() => {
    if (scrollerRef.current) {
      scrollerRef.current.scrollTop = 0;
    }
  }, [source.layout]);

  useEffect(() => revealLine(selectedIndex, ECodeLineReveal.NEAREST), [revealLine, selectedIndex]);

  useEffect(() => {
    if (scrollToLine !== null) {
      revealLine(source.indexOfLine(scrollToLine), ECodeLineReveal.START);
    }
  }, [revealLine, scrollToLine, source]);

  useEffect(() => {
    if (!onVisibleRangeChange) {
      return;
    }

    // The interval is clamped the way the rows themselves are: the virtualizer's last position is exclusive and may
    // sit past the end of a document that just got shorter.
    const first: number = Math.max(renderContext.firstRowIndex, 0);
    const last: number = Math.min(renderContext.lastRowIndex, count) - 1;

    if (last < first) {
      return;
    }

    const range: ICodeLineRange = {
      firstLine: source.getLine(first).number,
      lastLine: source.getLine(last).number,
    };
    const reported: Nullable<IReportedWindow> = reportedWindowRef.current;

    // A scroll that lands on the same window has nothing new to say, and a page of content arriving is not a scroll.
    // The document has to be part of the comparison and its line numbers cannot stand in for it: a resolved document
    // is numbered from one, so the first screen of every one of them is the same forty numbers over different content.
    if (
      reported &&
      reported.layout === source.layout &&
      reported.range.firstLine === range.firstLine &&
      reported.range.lastLine === range.lastLine
    ) {
      return;
    }

    reportedWindowRef.current = { layout: source.layout, range };
    onVisibleRangeChange(range);
  }, [count, onVisibleRangeChange, renderContext, source]);

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
