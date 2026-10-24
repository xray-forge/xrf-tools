import { describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { ConfigsDocument, ConfigsProjectDescriptor } from "@/core/ipc/types/xrf-app";
import { LtxAnchoredFinding } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsProblemsPanel } from "@/core/ltx/components/panels/ConfigsProblemsPanel";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { ConfigsSchemeService } from "@/core/ltx/services/scheme";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockRestoredConfigsProject } from "@/fixtures/utils/configs-project";
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
async function renderProblems(findings: Array<LtxAnchoredFinding>): Promise<{
  render: RenderResult;
  documentService: ConfigsDocumentService;
}> {
  const container: Container = mockContainer([
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsFindingsService,
    ConfigsResolvedService,
    ConfigsSchemeService,
  ]);
  const documentService: ConfigsDocumentService = container.get(ConfigsDocumentService);

  await mockRestoredConfigsProject(container.get(ConfigsProjectService));
  documentService.document = documentService.document.asReady({
    findings: [],
    structure: { entryPoints: [ENTRY], includes: [], parseError: null, path: ENTRY, rootEntries: [], sections: [] },
    text: { isNormalized: true, lines: [], path: ENTRY },
  } as ConfigsDocument);

  setMockInvokeResponses({
    // A provisioning restore finds the same project again.
    "plugin:configs|restore_project": mockSessionSnapshot({} as ConfigsProjectDescriptor, "session-1"),
    "plugin:configs|list_findings": findings,
    // What the jump reads: the config the finding names, which is not the one on screen.
    "plugin:configs|read_document": {
      findings: [],
      structure: { entryPoints: [ENTRY], includes: [], parseError: null, path: OTHER, rootEntries: [], sections: [] },
      text: { isNormalized: true, lines: ["[wpn_bad]"], path: OTHER },
    },
  });

  return { documentService, render: renderWithProviders(<ConfigsProblemsPanel />, { container }) };
}

describe("ConfigsProblemsPanel", () => {
  it("verifies the root only once it is on screen, and says what it found", async () => {
    const { render } = await renderProblems([SCHEME_FINDING]);

    await waitFor(() => expect(render.getByText(/unsigned 32 bit number/)).toBeInTheDocument());

    // The rule names the section, and the subject the file and line a person has to open.
    expect(render.getByText("scheme: wpn_bad")).toBeInTheDocument();
    expect(render.getByText("w_bad.ltx:8")).toBeInTheDocument();
  });

  it("opens the config a finding names, which is rarely the one on screen", async () => {
    const { render, documentService } = await renderProblems([SCHEME_FINDING]);

    await waitFor(() => expect(render.getByText("w_bad.ltx:8")).toBeInTheDocument());
    await userEvent.click(render.getByText("w_bad.ltx:8"));

    await waitFor(() => expect(documentService.selected).toBe(OTHER));

    // The line, which only the authored view can place: a resolved document holds no line of any config.
    expect(documentService.revealed).toEqual({ kind: "line", line: 8 });
    expect(documentService.mode).toBe("authored");
  });

  it("says what was checked when a root has nothing to report", async () => {
    const { render } = await renderProblems([]);

    await waitFor(() => expect(render.getByText(new RegExp("No problems found"))).toBeInTheDocument());
  });
});
