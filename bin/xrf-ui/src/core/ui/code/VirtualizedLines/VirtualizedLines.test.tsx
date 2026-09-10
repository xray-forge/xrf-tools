import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ESyntaxToken } from "@/core/syntax/lib";
import { CODE } from "@/core/theme/tokens";
import { ECodeLineMark, ICodeLine, ICodeLineRange, ICodeLineSource, toCodeLineSource } from "@/core/ui/code/code-line";
import { VirtualizedLines } from "@/core/ui/code/VirtualizedLines/VirtualizedLines";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

const VIEWPORT_HEIGHT: number = 400;

/**
 * A line whose text is one plain span, for a case that is not about colouring.
 *
 * @param number - Number the gutter shows.
 * @param text - Line text.
 * @param mark - Mark drawn beside the number.
 * @returns One line.
 */
function line(number: number, text: string, mark?: ECodeLineMark): ICodeLine {
  return { mark, number, spans: [{ token: ESyntaxToken.PLAIN, text }] };
}

/**
 * Lifts `@mui/x-virtualizer`'s own opt-out.
 *
 * It reads `platform.env.jsdom` once, when a virtualizer's store is created, and turns windowing off
 * under a test runner - which is why `VirtualizedTree` cannot assert its window and verifies it in the
 * running app instead. Flipped here because a bounded window is the only reason this component exists,
 * and a rewrite into one `<pre>` would otherwise pass every test in this file.
 *
 * @param isEnabled - Whether the virtualizer should behave as it does in a browser.
 */
function setVirtualizationEnabled(isEnabled: boolean): void {
  const platformModule: { platform: { env: { jsdom: boolean } } } = require("@base-ui/utils/platform");

  platformModule.platform.env.jsdom = !isEnabled;
}

/**
 * Gives the listing a viewport and a scroll position it can actually keep.
 *
 * jsdom lays nothing out: it answers `clientHeight` with zero and drops every write to `scrollTop`, so a component
 * that scrolls by arithmetic has nowhere to write its answer until these are its own properties.
 *
 * @param list - The listing's scroller.
 * @returns The same element, now scrollable.
 */
function asScrollable(list: HTMLElement): HTMLElement {
  Object.defineProperty(list, "clientHeight", { configurable: true, value: VIEWPORT_HEIGHT });
  Object.defineProperty(list, "scrollTop", { configurable: true, value: 0, writable: true });

  return list;
}

describe("VirtualizedLines", () => {
  it("exposes itself as a list of selectable lines", () => {
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} source={toCodeLineSource([line(1, "a"), line(2, "b")])} />
    );

    expect(render_.getByRole("listbox", { name: "Source" })).toBeInTheDocument();
    expect(render_.getAllByRole("option")).toHaveLength(2);
  });

  it("numbers the gutter from what a line says rather than from where it sits", () => {
    // An excerpt keeps the numbering of the file it came from, and a resolved document is assembled
    // rather than read off one, so the index is never the answer.
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} source={toCodeLineSource([line(40, "a"), line(41, "b"), line(99, "c")])} />
    );

    const gutters: Array<HTMLElement> = render_.getAllByTestId("virtualized-lines-gutter");

    expect(gutters.map((it: HTMLElement) => it.textContent)).toEqual(["40", "41", "99"]);
  });

  it("draws a mark beside the number of the line that carries it", () => {
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource([
          line(1, "a"),
          line(2, "b", ECodeLineMark.ERROR),
          line(3, "c", ECodeLineMark.WARNING),
        ])}
      />
    );

    expect(render_.getAllByTestId("virtualized-lines-gutter")[1]).toContainElement(render_.getByTitle("Error"));
    expect(render_.getAllByTestId("virtualized-lines-gutter")[2]).toContainElement(render_.getByTitle("Warning"));
    expect(render_.getAllByTestId("virtualized-lines-gutter")[0].querySelector("svg")).toBeNull();
  });

  it("renders the spans a line was built with, and colours only what is not plain", () => {
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource([
          {
            number: 1,
            spans: [
              { token: ESyntaxToken.KEY, text: "cost" },
              { token: ESyntaxToken.PLAIN, text: " = " },
              { token: ESyntaxToken.NUMBER, text: "1000" },
            ],
          },
        ])}
      />
    );

    const row: HTMLElement = render_.getAllByRole("option")[0];

    expect(row).toHaveTextContent("1cost = 1000");
    // Two coloured runs and the plain one left to inherit, which is most of a real file.
    expect(row.querySelectorAll("span[style*='color']")).toHaveLength(2);
  });

  it("reports the line a click chose, by the line itself and not its position", async () => {
    const onSelectLine = jest.fn();
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource([line(40, "a"), line(41, "b")])}
        onSelectLine={onSelectLine}
      />
    );

    await userEvent.click(render_.getByText("b"));

    expect(onSelectLine).toHaveBeenCalledWith(expect.objectContaining({ number: 41 }));
  });

  it("draws the selection on the line naming that number, wherever it sits", () => {
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource([line(40, "a"), line(41, "b")])}
        selectedLine={41}
      />
    );

    const rows: Array<HTMLElement> = render_.getAllByRole("option");

    expect(rows.map((it: HTMLElement) => it.getAttribute("aria-selected"))).toEqual(["false", "true"]);
    expect(render_.getByRole("listbox")).toHaveAttribute("aria-activedescendant", rows[1].id);
  });

  it("moves the selection with the arrow keys without moving focus off the list", async () => {
    const onSelectLine = jest.fn();
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource([line(40, "a"), line(41, "b")])}
        onSelectLine={onSelectLine}
      />
    );
    const list: HTMLElement = render_.getByRole("listbox");

    list.focus();
    await userEvent.keyboard("{ArrowDown}{End}");

    expect(list).toHaveFocus();
    expect(onSelectLine).toHaveBeenLastCalledWith(expect.objectContaining({ number: 41 }));
  });

  it("puts a line it was addressed to at the top of the viewport", () => {
    // A section jumped to is read downwards from its header, so scrolling just far enough to make it visible is the
    // wrong answer: it would sit against the bottom edge, showing its name and none of its body.
    const lines: Array<ICodeLine> = Array.from({ length: 300 }, (_, index: number) => line(index + 1, "a"));
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} source={toCodeLineSource(lines)} />
    );
    const list: HTMLElement = asScrollable(render_.getByRole("listbox"));

    render_.rerender(
      <>
        <VirtualizedLines ariaLabel={"Source"} source={toCodeLineSource(lines)} scrollToLine={200} />
      </>
    );

    expect(list.scrollTop).toBe(199 * CODE.lineHeight);
  });

  it("moves the listing as little as it can for a step of the selection", () => {
    // The other half of the same decision: an arrow key that parked its line at the top would throw away the context
    // the reader is looking at, so a step still scrolls by the least it can.
    const lines: Array<ICodeLine> = Array.from({ length: 300 }, (_, index: number) => line(index + 1, "a"));
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} source={toCodeLineSource(lines)} />
    );
    const list: HTMLElement = asScrollable(render_.getByRole("listbox"));

    render_.rerender(
      <>
        <VirtualizedLines ariaLabel={"Source"} source={toCodeLineSource(lines)} selectedLine={300} />
      </>
    );

    expect(list.scrollTop).toBe(300 * CODE.lineHeight - VIEWPORT_HEIGHT);
  });

  it("reports the stretch on screen by the numbers its lines show, not by their positions", () => {
    // What a listing whose content is fetched as it scrolls asks for, and it asks in the same currency everything
    // else here is addressed in.
    const onVisibleRangeChange = jest.fn();

    renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource([line(40, "a"), line(41, "b"), line(99, "c")])}
        onVisibleRangeChange={onVisibleRangeChange}
      />
    );

    expect(onVisibleRangeChange).toHaveBeenCalledTimes(1);
    expect(onVisibleRangeChange).toHaveBeenCalledWith({ firstLine: 40, lastLine: 99 });
  });

  it("stays quiet while the stretch on screen does not change", () => {
    // A fetch driven by this callback would otherwise be asked for the same page on every scroll frame.
    const onVisibleRangeChange = jest.fn();
    const lines: Array<ICodeLine> = [line(1, "a"), line(2, "b")];
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource(lines)}
        onVisibleRangeChange={onVisibleRangeChange}
      />
    );

    // Re-wrapped exactly as `renderWithProviders` wraps the first render: a rerender that changes the shape of the
    // tree above the subject remounts it, which would reset what it remembers reporting and prove nothing.
    render_.rerender(
      <>
        <VirtualizedLines
          ariaLabel={"Source"}
          source={toCodeLineSource(lines)}
          selectedLine={2}
          // A fresh function every render, which is what a caller writing the handler inline hands over.
          onVisibleRangeChange={(range: ICodeLineRange) => onVisibleRangeChange(range)}
        />
      </>
    );

    expect(onVisibleRangeChange).toHaveBeenCalledTimes(1);
  });

  it("reports again once the document under it is numbered differently", () => {
    const onVisibleRangeChange = jest.fn();
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource([line(1, "a"), line(2, "b")])}
        onVisibleRangeChange={onVisibleRangeChange}
      />
    );

    render_.rerender(
      <>
        <VirtualizedLines
          ariaLabel={"Source"}
          source={toCodeLineSource([line(400, "a"), line(401, "b")])}
          onVisibleRangeChange={onVisibleRangeChange}
        />
      </>
    );

    expect(onVisibleRangeChange).toHaveBeenCalledTimes(2);
    expect(onVisibleRangeChange).toHaveBeenLastCalledWith({ firstLine: 400, lastLine: 401 });
  });

  it("measures its scroll height from the line count instead of laying the lines out", () => {
    // The signature of a windowed listing: the scroller learns how tall the document is by arithmetic,
    // so the rows it never rendered still take up the room they would have.
    const lines: Array<ICodeLine> = Array.from({ length: 4000 }, (_, index: number) => line(index + 1, "a"));
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} source={toCodeLineSource(lines)} />
    );
    const sizer: Nullable<HTMLElement> = render_.getByRole("listbox").querySelector("div[role='presentation']");

    expect(sizer).toHaveStyle({ height: `${4000 * CODE.lineHeight}px` });
  });
});

describe("VirtualizedLines with windowing on", () => {
  beforeEach(() => {
    setVirtualizationEnabled(true);

    // jsdom lays nothing out, so the scroller would report no viewport and the window would be empty.
    jest.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
      bottom: VIEWPORT_HEIGHT,
      height: VIEWPORT_HEIGHT,
      left: 0,
      right: 800,
      top: 0,
      width: 800,
    } as DOMRect);
  });

  afterEach(() => {
    setVirtualizationEnabled(false);
    jest.restoreAllMocks();
  });

  it("renders a window of lines rather than the document behind it", () => {
    const lines: Array<ICodeLine> = Array.from({ length: 300000 }, (_, index: number) => line(index + 1, "a"));
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} source={toCodeLineSource(lines)} />
    );
    const rows: Array<HTMLElement> = render_.getAllByRole("option");

    // What is asserted is the shape of the cost, not an exact count: the window is the viewport plus
    // the virtualizer's own scroll buffer, and it is a function of neither the 300,000 lines behind it
    // nor of anything this component holds.
    expect(rows.length).toBeGreaterThan(0);
    expect(rows.length).toBeLessThan((3 * VIEWPORT_HEIGHT) / CODE.lineHeight);
    expect(render_.getAllByTestId("virtualized-lines-gutter")[0]).toHaveTextContent("1");
  });

  it("reports the window it rendered rather than the document behind it", () => {
    // The report is what a viewport-driven fetch is sized by, so it has to be the window and not the whole answer.
    const onVisibleRangeChange = jest.fn();
    const lines: Array<ICodeLine> = Array.from({ length: 300000 }, (_, index: number) => line(index + 1, "a"));

    renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource(lines)}
        onVisibleRangeChange={onVisibleRangeChange}
      />
    );

    const range: { firstLine: number; lastLine: number } = onVisibleRangeChange.mock.calls[0][0] as ICodeLineRange;

    expect(range.firstLine).toBe(1);
    expect(range.lastLine).toBeLessThan(lines.length);
  });

  it("reports again when the document changed under a window numbered the same", () => {
    // Every resolved document is numbered from one, so selecting another config shows different content behind the
    // same numbers. Staying quiet there leaves the new document's first screen blank until the reader scrolls, which
    // is the one thing the report exists to prevent.
    const onVisibleRangeChange = jest.fn();
    const first: Array<ICodeLine> = Array.from({ length: 300000 }, (_, index: number) => line(index + 1, "a"));
    const second: Array<ICodeLine> = Array.from({ length: 300000 }, (_, index: number) => line(index + 1, "b"));
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        source={toCodeLineSource(first)}
        onVisibleRangeChange={onVisibleRangeChange}
      />
    );

    render_.rerender(
      <>
        <VirtualizedLines
          ariaLabel={"Source"}
          source={toCodeLineSource(second)}
          onVisibleRangeChange={onVisibleRangeChange}
        />
      </>
    );

    expect(onVisibleRangeChange).toHaveBeenCalledTimes(2);
  });

  it("asks its source only for the lines it draws", () => {
    // Why the listing takes a source instead of an array. Anomaly's resolved `system.ltx` is 551,000 lines, and every
    // page of section bodies that lands changes what some of them say: a listing that materialised the document would
    // rebuild half a million lines per page, for a screen that shows forty.
    const asked: Set<number> = new Set();
    const source: ICodeLineSource = {
      count: 300000,
      getLine: (index: number): ICodeLine => {
        asked.add(index);

        return line(index + 1, "a");
      },
      indexOfLine: (number: number): number => number - 1,
      layout: {},
      widestNumber: 300000,
    };

    renderWithProviders(<VirtualizedLines ariaLabel={"Source"} source={source} />);

    expect(asked.size).toBeGreaterThan(0);
    expect(asked.size).toBeLessThan((3 * VIEWPORT_HEIGHT) / CODE.lineHeight);
  });
});
