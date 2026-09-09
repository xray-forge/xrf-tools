import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ESyntaxToken } from "@/core/syntax/lib";
import { CODE } from "@/core/theme/tokens";
import { ECodeLineMark, ICodeLine } from "@/core/ui/code/code-line";
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

describe("VirtualizedLines", () => {
  it("exposes itself as a list of selectable lines", () => {
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} lines={[line(1, "a"), line(2, "b")]} />
    );

    expect(render_.getByRole("listbox", { name: "Source" })).toBeInTheDocument();
    expect(render_.getAllByRole("option")).toHaveLength(2);
  });

  it("numbers the gutter from what a line says rather than from where it sits", () => {
    // An excerpt keeps the numbering of the file it came from, and a resolved document is assembled
    // rather than read off one, so the index is never the answer.
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} lines={[line(40, "a"), line(41, "b"), line(99, "c")]} />
    );

    const gutters: Array<HTMLElement> = render_.getAllByTestId("virtualized-lines-gutter");

    expect(gutters.map((it: HTMLElement) => it.textContent)).toEqual(["40", "41", "99"]);
  });

  it("draws a mark beside the number of the line that carries it", () => {
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines
        ariaLabel={"Source"}
        lines={[line(1, "a"), line(2, "b", ECodeLineMark.ERROR), line(3, "c", ECodeLineMark.WARNING)]}
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
        lines={[
          {
            number: 1,
            spans: [
              { token: ESyntaxToken.KEY, text: "cost" },
              { token: ESyntaxToken.PLAIN, text: " = " },
              { token: ESyntaxToken.NUMBER, text: "1000" },
            ],
          },
        ]}
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
      <VirtualizedLines ariaLabel={"Source"} lines={[line(40, "a"), line(41, "b")]} onSelectLine={onSelectLine} />
    );

    await userEvent.click(render_.getByText("b"));

    expect(onSelectLine).toHaveBeenCalledWith(expect.objectContaining({ number: 41 }));
  });

  it("draws the selection on the line naming that number, wherever it sits", () => {
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} lines={[line(40, "a"), line(41, "b")]} selectedLine={41} />
    );

    const rows: Array<HTMLElement> = render_.getAllByRole("option");

    expect(rows.map((it: HTMLElement) => it.getAttribute("aria-selected"))).toEqual(["false", "true"]);
    expect(render_.getByRole("listbox")).toHaveAttribute("aria-activedescendant", rows[1].id);
  });

  it("moves the selection with the arrow keys without moving focus off the list", async () => {
    const onSelectLine = jest.fn();
    const render_: RenderResult = renderWithProviders(
      <VirtualizedLines ariaLabel={"Source"} lines={[line(40, "a"), line(41, "b")]} onSelectLine={onSelectLine} />
    );
    const list: HTMLElement = render_.getByRole("listbox");

    list.focus();
    await userEvent.keyboard("{ArrowDown}{End}");

    expect(list).toHaveFocus();
    expect(onSelectLine).toHaveBeenLastCalledWith(expect.objectContaining({ number: 41 }));
  });

  it("measures its scroll height from the line count instead of laying the lines out", () => {
    // The signature of a windowed listing: the scroller learns how tall the document is by arithmetic,
    // so the rows it never rendered still take up the room they would have.
    const lines: Array<ICodeLine> = Array.from({ length: 4000 }, (_, index: number) => line(index + 1, "a"));
    const render_: RenderResult = renderWithProviders(<VirtualizedLines ariaLabel={"Source"} lines={lines} />);
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
    const render_: RenderResult = renderWithProviders(<VirtualizedLines ariaLabel={"Source"} lines={lines} />);
    const rows: Array<HTMLElement> = render_.getAllByRole("option");

    // What is asserted is the shape of the cost, not an exact count: the window is the viewport plus
    // the virtualizer's own scroll buffer, and it is a function of neither the 300,000 lines behind it
    // nor of anything this component holds.
    expect(rows.length).toBeGreaterThan(0);
    expect(rows.length).toBeLessThan((3 * VIEWPORT_HEIGHT) / CODE.lineHeight);
    expect(render_.getAllByTestId("virtualized-lines-gutter")[0]).toHaveTextContent("1");
  });
});
