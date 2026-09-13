import { describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { ConfigsDocument } from "@/core/ipc/types/xrf-app";
import { LtxAnchoredFinding } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsProblemsPanel } from "@/core/ltx/components/panels/ConfigsProblemsPanel";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

const ENTRY: string = "configs\\system.ltx";
const OTHER: string = "configs\\items\\w_bad.ltx";

/** A finding of the root, anchored where the section is written. */
const SCHEME_FINDING: LtxAnchoredFinding = {
  engineBehaviour: null,
  entry: ENTRY,
  field: "cost",
  file: OTHER,
  kind: "scheme",
  line: 8,
  message: "Invalid value, unsigned 32 bit number is expected, got 'expensive'",
  section: "wpn_bad",
};

/** The panel with one config open, over whatever the root's verification answers. */
function renderProblems(findings: Array<LtxAnchoredFinding>): {
  render: RenderResult;
  documentService: ConfigsDocumentService;
} {
  const container: Container = mockContainer([ConfigsProjectService, ConfigsDocumentService, ConfigsFindingsService]);
  const project = container.get(ConfigsProjectService);
  const documentService: ConfigsDocumentService = container.get(ConfigsDocumentService);

  project.project = project.project.asReady({ sessionId: "session-1" } as never);
  documentService.document = documentService.document.asReady({
    findings: [],
    structure: { entryPoints: [ENTRY], includes: [], parseError: null, path: ENTRY, sections: [] },
    text: { isNormalized: true, lines: [], path: ENTRY },
  } as ConfigsDocument);

  setMockInvokeResponses({
    "plugin:configs|get_project": project.project.value,
    "plugin:configs|list_findings": findings,
    // What the jump reads: the config the finding names, which is not the one on screen.
    "plugin:configs|read_document": {
      findings: [],
      structure: { entryPoints: [ENTRY], includes: [], parseError: null, path: OTHER, sections: [] },
      text: { isNormalized: true, lines: ["[wpn_bad]"], path: OTHER },
    },
  });

  return { documentService, render: renderWithProviders(<ConfigsProblemsPanel />, { container }) };
}

describe("ConfigsProblemsPanel", () => {
  it("verifies the root only once it is on screen, and says what it found", async () => {
    const { render } = renderProblems([SCHEME_FINDING]);

    await waitFor(() => expect(render.getByText(/unsigned 32 bit number/)).toBeInTheDocument());

    // The rule names the section, and the subject the file and line a person has to open.
    expect(render.getByText("scheme: wpn_bad")).toBeInTheDocument();
    expect(render.getByText("w_bad.ltx:8")).toBeInTheDocument();
  });

  it("opens the config a finding names, which is rarely the one on screen", async () => {
    const { render, documentService } = renderProblems([SCHEME_FINDING]);

    await waitFor(() => expect(render.getByText("w_bad.ltx:8")).toBeInTheDocument());
    await userEvent.click(render.getByText("w_bad.ltx:8"));

    await waitFor(() => expect(documentService.selected).toBe(OTHER));

    // The line, which only the authored view can place: a resolved document holds no line of any config.
    expect(documentService.revealed).toEqual({ kind: "line", line: 8 });
    expect(documentService.mode).toBe("authored");
  });

  it("says what was checked when a root has nothing to report", async () => {
    const { render } = renderProblems([]);

    await waitFor(() => expect(render.getByText(new RegExp("No problems found"))).toBeInTheDocument());
  });
});
