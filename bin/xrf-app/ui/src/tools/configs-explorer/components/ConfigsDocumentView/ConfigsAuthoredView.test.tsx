import { describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { ConfigsDocument } from "@xrf/ipc/types/xrf-app-configs";
import { LtxAnchoredFinding } from "@xrf/ipc/types/xrf-ltx-inspect";

import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { ConfigsAuthoredView } from "@/tools/configs-explorer/components/ConfigsDocumentView/ConfigsAuthoredView";
import { ConfigsDocumentService } from "@/tools/configs-explorer/services/configs-document";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/tools/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/tools/configs-explorer/services/configs-scheme";

const PATH: string = "configs\\items\\w_empty.ltx";

/** A document carrying the lines a case is about and a structure that says nothing else. */
function documentOf(lines: Array<string>): ConfigsDocument {
  return {
    findings: [],
    structure: { entryPoints: [], includes: [], parseError: null, path: PATH, rootEntries: [], sections: [] },
    text: { isNormalized: true, lines, path: PATH },
  };
}

function renderAuthored(document: ConfigsDocument): { render: RenderResult; container: Container } {
  const container: Container = mockContainer([
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsFindingsService,
    ConfigsResolvedService,
    ConfigsSchemeService,
  ]);

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
    const {
      render: { getByText, queryAllByRole },
    } = renderAuthored(documentOf([]));

    expect(getByText("This config is empty")).toBeInTheDocument();
    expect(getByText(new RegExp(PATH.replace(/\\/g, "\\\\")))).toBeInTheDocument();
    expect(queryAllByRole("option")).toHaveLength(0);
  });

  it("draws the file when it has one, including the empty line its last newline starts", () => {
    const {
      render: { getAllByRole, getAllByTestId, queryByText },
    } = renderAuthored(documentOf(["[wpn_base]", "cost = 100", ""]));

    expect(queryByText("This config is empty")).not.toBeInTheDocument();
    expect(getAllByRole("option")).toHaveLength(3);
    expect(getAllByTestId("virtualized-lines-gutter").map((it: HTMLElement) => it.textContent)).toEqual([
      "1",
      "2",
      "3",
    ]);
  });

  it("marks the gutter when the root's findings arrive after the config did", () => {
    // The order a reader actually produces: a config opens, and the Problems panel verifies the root afterwards. A
    // view that only read the findings it had at open would leave the file it is showing unmarked.
    const {
      render: { getAllByTestId, getByTitle, queryByTitle },
      container,
    } = renderAuthored(documentOf(["[wpn_base]", "cost = expensive"]));
    const findingsService: ConfigsFindingsService = container.get(ConfigsFindingsService);

    expect(queryByTitle("Error")).not.toBeInTheDocument();

    act(() => runInAction(() => (findingsService.findings = findingsService.findings.asReady([FINDING]))));

    expect(getAllByTestId("virtualized-lines-gutter")[1]).toContainElement(getByTitle("Error"));
  });
});
