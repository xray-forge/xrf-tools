import { describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ConfigsDocument } from "@/core/ipc/types/xrf-app";
import { LtxAnchoredFinding } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsAuthoredView } from "@/core/ltx/components/ConfigsDocumentView/ConfigsAuthoredView";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

const PATH: string = "configs\\items\\w_empty.ltx";

/** A document carrying the lines a case is about and a structure that says nothing else. */
function documentOf(lines: Array<string>): ConfigsDocument {
  return {
    findings: [],
    structure: { entryPoints: [], includes: [], parseError: null, path: PATH, sections: [] },
    text: { isNormalized: true, lines, path: PATH },
  };
}

function renderAuthored(document: ConfigsDocument): { render: RenderResult; container: Container } {
  const container: Container = mockContainer([ConfigsProjectService, ConfigsDocumentService, ConfigsFindingsService]);

  return { container, render: renderWithProviders(<ConfigsAuthoredView document={document} />, { container }) };
}

/** One finding of the root, anchored in the config being drawn. */
const FINDING: LtxAnchoredFinding = {
  engineBehaviour: null,
  entry: PATH,
  field: "cost",
  file: PATH,
  kind: "scheme",
  line: 2,
  message: "Invalid value",
  section: "wpn_base",
};

describe("ConfigsAuthoredView", () => {
  it("says a config is empty rather than drawing a listing of nothing", () => {
    // A mod that empties a vanilla file ships it empty rather than deleting it, so this is a real file to open - and
    // an empty listing is indistinguishable from one that failed to render.
    const { render: render_ } = renderAuthored(documentOf([]));

    expect(render_.getByText("This config is empty")).toBeInTheDocument();
    expect(render_.getByText(new RegExp(PATH.replace(/\\/g, "\\\\")))).toBeInTheDocument();
    expect(render_.queryAllByRole("option")).toHaveLength(0);
  });

  it("draws the file when it has one, including the empty line its last newline starts", () => {
    const { render: render_ } = renderAuthored(documentOf(["[wpn_base]", "cost = 100", ""]));

    expect(render_.queryByText("This config is empty")).not.toBeInTheDocument();
    expect(render_.getAllByRole("option")).toHaveLength(3);
    expect(render_.getAllByTestId("virtualized-lines-gutter").map((it: HTMLElement) => it.textContent)).toEqual([
      "1",
      "2",
      "3",
    ]);
  });

  it("marks the gutter when the root's findings arrive after the config did", () => {
    // The order a reader actually produces: a config opens, and the Problems panel verifies the root afterwards. A
    // view that only read the findings it had at open would leave the file it is showing unmarked.
    const { render: render_, container } = renderAuthored(documentOf(["[wpn_base]", "cost = expensive"]));
    const findingsService: ConfigsFindingsService = container.get(ConfigsFindingsService);

    expect(render_.queryByTitle("Error")).not.toBeInTheDocument();

    act(() => runInAction(() => (findingsService.findings = findingsService.findings.asReady([FINDING]))));

    expect(render_.getAllByTestId("virtualized-lines-gutter")[1]).toContainElement(render_.getByTitle("Error"));
  });
});
