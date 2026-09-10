import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { ConfigsDocument } from "@/core/bindings/types/xrf-app";
import { ConfigsAuthoredView } from "@/core/ltx/components/ConfigsDocumentView/ConfigsAuthoredView";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

const PATH: string = "configs\\items\\w_empty.ltx";

/** A document carrying the lines a case is about and a structure that says nothing else. */
function documentOf(lines: Array<string>): ConfigsDocument {
  return {
    structure: { entryPoints: [], includes: [], parseError: null, path: PATH, sections: [] },
    text: { isNormalized: true, lines, path: PATH },
  };
}

function renderAuthored(document: ConfigsDocument): RenderResult {
  const container: Container = mockContainer([ConfigsProjectService, ConfigsDocumentService]);

  return renderWithProviders(<ConfigsAuthoredView document={document} />, { container });
}

describe("ConfigsAuthoredView", () => {
  it("says a config is empty rather than drawing a listing of nothing", () => {
    // A mod that empties a vanilla file ships it empty rather than deleting it, so this is a real file to open - and
    // an empty listing is indistinguishable from one that failed to render.
    const render_: RenderResult = renderAuthored(documentOf([]));

    expect(render_.getByText("This config is empty")).toBeInTheDocument();
    expect(render_.getByText(new RegExp(PATH.replace(/\\/g, "\\\\")))).toBeInTheDocument();
    expect(render_.queryAllByRole("option")).toHaveLength(0);
  });

  it("draws the file when it has one, including the empty line its last newline starts", () => {
    const render_: RenderResult = renderAuthored(documentOf(["[wpn_base]", "cost = 100", ""]));

    expect(render_.queryByText("This config is empty")).not.toBeInTheDocument();
    expect(render_.getAllByRole("option")).toHaveLength(3);
    expect(render_.getAllByTestId("virtualized-lines-gutter").map((it: HTMLElement) => it.textContent)).toEqual([
      "1",
      "2",
      "3",
    ]);
  });
});
