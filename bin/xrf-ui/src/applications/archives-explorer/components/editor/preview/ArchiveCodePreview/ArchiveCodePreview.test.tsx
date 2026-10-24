import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ArchiveReadResult } from "@xrf/ipc/types/xrf-archive";
import { ESyntaxToken } from "@xrf/syntax";
import { Nullable } from "@xrf/types";

import { renderWithProviders } from "@/fixtures/utils/render";
import { setVirtualizationEnabled } from "@/fixtures/utils/virtualization";

import { ArchiveCodePreview } from "./ArchiveCodePreview";

const VIEWPORT_HEIGHT: number = 400;

function getReadOf(name: string, content: string): ArchiveReadResult {
  return { content, name, size: content.length };
}

function asScrollable(list: HTMLElement): HTMLElement {
  Object.defineProperty(list, "clientHeight", { configurable: true, value: VIEWPORT_HEIGHT });
  Object.defineProperty(list, "scrollTop", { configurable: true, value: 0, writable: true });

  return list;
}

describe("ArchiveCodePreview", () => {
  it("colours a config by its grammar", () => {
    const { container } = renderWithProviders(
      <ArchiveCodePreview file={getReadOf("textures.ltx", "[association]\n\tact\\act_lenin = detail\\det3, 6.0")} />
    );

    const coloured: Array<Nullable<string>> = Array.from(
      container.querySelectorAll<HTMLSpanElement>(`[data-syntax-token="${ESyntaxToken.SECTION}"]`)
    ).map((span: HTMLSpanElement) => span.textContent);

    expect(coloured).toEqual(["[association]"]);
  });

  it("numbers every line of the file it was handed", () => {
    const { getAllByRole, getByRole } = renderWithProviders(
      <ArchiveCodePreview file={getReadOf("a.ltx", "[a]\nkey = 1\n[b]")} />
    );

    expect(getAllByRole("option")).toHaveLength(3);
    expect(getByRole("listbox", { name: "Contents of a.ltx" })).toBeInTheDocument();
  });

  it("marks the line a reader clicked, and drops it when another file opens", async () => {
    // The preview writes nothing back, so the selection exists only to keep a place - and a line number outliving
    // the file it belonged to would point at an unrelated line of the next one.
    const { getAllByRole, rerender } = renderWithProviders(
      <ArchiveCodePreview file={getReadOf("a.ltx", "[a]\nkey = 1\n[b]")} />
    );

    await userEvent.click(getAllByRole("option")[1]);

    expect(getAllByRole("option")[1]).toHaveAttribute("aria-selected", "true");

    rerender(<ArchiveCodePreview file={getReadOf("b.ltx", "[c]\nkey = 2\n[d]")} />);

    expect(getAllByRole("option").filter((row: HTMLElement) => row.getAttribute("aria-selected") === "true")).toEqual(
      []
    );
  });

  describe("with windowing on", () => {
    beforeEach(() => setVirtualizationEnabled(true));

    afterEach(() => setVirtualizationEnabled(false));

    it("draws a window of a file far past what colouring one in a single pass would be worth", () => {
      // The case this replaced `CodeView` for: Anomaly's `textures.ltx` is some 700 KB, which the old preview
      // rendered as one `pre` of tens of thousands of nodes and gave up colouring entirely.
      const content: string = new Array(60_000).fill("act\\act_lenin = detail\\detail_tile_det3, 6.000000").join("\n");
      const { container, getAllByRole, getByRole } = renderWithProviders(
        <ArchiveCodePreview file={getReadOf("textures.ltx", content)} />
      );

      asScrollable(getByRole("listbox"));

      expect(content.length).toBeGreaterThan(512 * 1024);
      expect(getAllByRole("option").length).toBeLessThan(60);
      expect(container.querySelectorAll(`[data-syntax-token="${ESyntaxToken.KEY}"]`).length).toBeGreaterThan(0);
    });
  });
});
