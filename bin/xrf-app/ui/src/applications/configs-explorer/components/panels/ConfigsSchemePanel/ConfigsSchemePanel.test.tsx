import { describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { ConfigsDocument } from "@xrf/ipc/types/xrf-app-configs";
import { LtxSchemeFieldReport, LtxSectionSchemeReport } from "@xrf/ipc/types/xrf-ltx-inspect";

import { ConfigsSchemePanel } from "@/applications/configs-explorer/components/panels/ConfigsSchemePanel/ConfigsSchemePanel";
import { mockRestoredConfigsProject } from "@/applications/configs-explorer/fixtures/configs-project";
import { ConfigsDocumentService } from "@/applications/configs-explorer/services/document";
import { ConfigsFindingsService } from "@/applications/configs-explorer/services/findings";
import { ConfigsProjectService } from "@/applications/configs-explorer/services/project";
import { ConfigsResolvedService } from "@/applications/configs-explorer/services/resolved";
import { ConfigsSchemeService } from "@/applications/configs-explorer/services/scheme";
import { mockSessionRead } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

const ENTRY: string = "configs\\system.ltx";

/** A field row carrying only what a case is about. */
function fieldOf(name: string, declared: LtxSchemeFieldReport["declared"], value: string | null): LtxSchemeFieldReport {
  return {
    declared,
    name,
    resolved: value === null ? null : { key: name, origin: { file: null, kind: "declared" }, value },
  };
}

/** The panel over one report, with a section already selected. */
async function renderScheme(report: LtxSectionSchemeReport): Promise<RenderResult> {
  const container: Container = mockContainer([
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsSchemeService,
    ConfigsFindingsService,
    ConfigsResolvedService,
  ]);
  const documentService: ConfigsDocumentService = container.get(ConfigsDocumentService);

  await mockRestoredConfigsProject(container.get(ConfigsProjectService));
  documentService.document = documentService.document.asReady({
    findings: [],
    structure: { entryPoints: [ENTRY], includes: [], parseError: null, path: ENTRY, rootEntries: [], sections: [] },
    text: { isNormalized: true, lines: [], path: ENTRY },
  } as ConfigsDocument);
  documentService.selectSection(report.section);

  mockIpcCommands({ [EConfigsCommand.READ_SECTION_SCHEME]: mockSessionRead(report) });

  return renderWithProviders(<ConfigsSchemePanel />, { container });
}

describe("ConfigsSchemePanel", () => {
  it("names the rule, where the binding is written, and what each field is asked for", async () => {
    const { getByText } = await renderScheme({
      entry: ENTRY,
      fields: [
        fieldOf("cost", { dataType: "u32", isAny: false, isArray: false, isOptional: false }, "4000"),
        fieldOf("description", { dataType: "string", isAny: false, isArray: false, isOptional: true }, null),
      ],
      inheritedFrom: "wpn_base",
      isDeclared: true,
      isStrict: true,
      scheme: "$weapon",
      section: "wpn_child",
    });

    await waitFor(() => expect(getByText("$weapon")).toBeInTheDocument());

    // The answer nothing in the text of `[wpn_child]:wpn_base` gives.
    expect(getByText("inherited from wpn_base")).toBeInTheDocument();
    expect(getByText("strict")).toBeInTheDocument();
    expect(getByText("u32")).toBeInTheDocument();
    expect(getByText("4000")).toBeInTheDocument();
    // Declared, optional, and never supplied - which under a strict scheme is still not a finding.
    expect(getByText("string?")).toBeInTheDocument();
    expect(getByText("not supplied")).toBeInTheDocument();
  });

  it("draws plain rows for a section no scheme judges", async () => {
    // Most of a game tree: section schemes are an XRF convention, so nothing is undeclared against nothing.
    const { getByText, queryByText } = await renderScheme({
      entry: ENTRY,
      fields: [fieldOf("cost", null, "100")],
      inheritedFrom: null,
      isDeclared: false,
      isStrict: false,
      scheme: null,
      section: "wpn_plain",
    });

    await waitFor(() => expect(getByText("no scheme")).toBeInTheDocument());

    expect(getByText("100")).toBeInTheDocument();
    expect(queryByText("undeclared")).not.toBeInTheDocument();
  });

  it("marks a field the scheme never named, once a scheme is judging the section", async () => {
    const { getByText } = await renderScheme({
      entry: ENTRY,
      fields: [fieldOf("unexpected_field", null, "1")],
      inheritedFrom: null,
      isDeclared: true,
      isStrict: true,
      scheme: "$weapon",
      section: "wpn_broken",
    });

    await waitFor(() => expect(getByText("undeclared")).toBeInTheDocument());
  });
});
