import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ArchiveReadResult } from "@/core/ipc/types/xrf-archive";
import { ESyntaxToken } from "@/core/syntax/lib";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveCodePreview } from "./ArchiveCodePreview";

const VIEWPORT_HEIGHT: number = 400;

function getReadOf(name: string, content: string): ArchiveReadResult {
  return { content, name, size: content.length };
}

function setVirtualizationEnabled(isEnabled: boolean): void {
  const platformModule: { platform: { env: { jsdom: boolean } } } = require("@base-ui/utils/platform");

  platformModule.platform.env.jsdom = !isEnabled;
}

function asScrollable(list: HTMLElement): HTMLElement {
  Object.defineProperty(list, "clientHeight", { configurable: true, value: VIEWPORT_HEIGHT });
  Object.defineProperty(list, "scrollTop", { configurable: true, value: 0, writable: true });

  return list;
}

describe("ArchiveCodePreview", () => {
  it("colours a config by its grammar", () => {
    const render_: RenderResult = renderWithProviders(
      <ArchiveCodePreview file={getReadOf("textures.ltx", "[association]\n\tact\\act_lenin = detail\\det3, 6.0")} />
    );

    const coloured: Array<string | null> = Array.from(
      render_.container.querySelectorAll<HTMLSpanElement>(`[data-syntax-token="${ESyntaxToken.SECTION}"]`)
    ).map((span: HTMLSpanElement) => span.textContent);

    expect(coloured).toEqual(["[association]"]);
  });

  it("numbers every line of the file it was handed", () => {
    const render_: RenderResult = renderWithProviders(
      <ArchiveCodePreview file={getReadOf("a.ltx", "[a]\nkey = 1\n[b]")} />
    );

    expect(render_.getAllByRole("option")).toHaveLength(3);
    expect(render_.getByRole("listbox", { name: "Contents of a.ltx" })).toBeInTheDocument();
  });

  it("marks the line a reader clicked, and drops it when another file opens", async () => {
    // The preview writes nothing back, so the selection exists only to keep a place - and a line number outliving
    // the file it belonged to would point at an unrelated line of the next one.
    const render_: RenderResult = renderWithProviders(
      <ArchiveCodePreview file={getReadOf("a.ltx", "[a]\nkey = 1\n[b]")} />
    );

    await userEvent.click(render_.getAllByRole("option")[1]);

    expect(render_.getAllByRole("option")[1]).toHaveAttribute("aria-selected", "true");

    render_.rerender(<ArchiveCodePreview file={getReadOf("b.ltx", "[c]\nkey = 2\n[d]")} />);

    expect(
      render_.getAllByRole("option").filter((row: HTMLElement) => row.getAttribute("aria-selected") === "true")
    ).toEqual([]);
  });

  describe("with windowing on", () => {
    beforeEach(() => setVirtualizationEnabled(true));

    afterEach(() => setVirtualizationEnabled(false));

    it("draws a window of a file far past what colouring one in a single pass would be worth", () => {
      // The case this replaced `CodeView` for: Anomaly's `textures.ltx` is some 700 KB, which the old preview
      // rendered as one `pre` of tens of thousands of nodes and gave up colouring entirely.
      const content: string = new Array(60_000).fill("act\\act_lenin = detail\\detail_tile_det3, 6.000000").join("\n");
      const render_: RenderResult = renderWithProviders(
        <ArchiveCodePreview file={getReadOf("textures.ltx", content)} />
      );

      asScrollable(render_.getByRole("listbox"));

      expect(content.length).toBeGreaterThan(512 * 1024);
      expect(render_.getAllByRole("option").length).toBeLessThan(60);
      expect(render_.container.querySelectorAll(`[data-syntax-token="${ESyntaxToken.KEY}"]`).length).toBeGreaterThan(0);
    });
  });
});
