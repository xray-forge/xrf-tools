import { describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { ConfigsDocument } from "@/core/bindings/types/xrf-app";
import { LtxSchemeFieldReport, LtxSectionSchemeReport } from "@/core/bindings/types/xrf-ltx-inspect";
import { ConfigsSchemePanel } from "@/core/ltx/components/panels/ConfigsSchemePanel/ConfigsSchemePanel";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsSchemeService } from "@/core/ltx/services/scheme";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
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
function renderScheme(report: LtxSectionSchemeReport): RenderResult {
  const container: Container = mockContainer([ConfigsProjectService, ConfigsDocumentService, ConfigsSchemeService]);
  const project = container.get(ConfigsProjectService);
  const documentService: ConfigsDocumentService = container.get(ConfigsDocumentService);

  project.project = project.project.asReady({ sessionId: "session-1" } as never);
  documentService.document = documentService.document.asReady({
    findings: [],
    structure: { entryPoints: [ENTRY], includes: [], parseError: null, path: ENTRY, sections: [] },
    text: { isNormalized: true, lines: [], path: ENTRY },
  } as ConfigsDocument);
  documentService.selectSection(report.section);

  setMockInvokeResponses({ "plugin:configs|read_section_scheme": report });

  return renderWithProviders(<ConfigsSchemePanel />, { container });
}

describe("ConfigsSchemePanel", () => {
  it("names the rule, where the binding is written, and what each field is asked for", async () => {
    const render_: RenderResult = renderScheme({
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

    await waitFor(() => expect(render_.getByText("$weapon")).toBeInTheDocument());

    // The answer nothing in the text of `[wpn_child]:wpn_base` gives.
    expect(render_.getByText("inherited from wpn_base")).toBeInTheDocument();
    expect(render_.getByText("strict")).toBeInTheDocument();
    expect(render_.getByText("u32")).toBeInTheDocument();
    expect(render_.getByText("4000")).toBeInTheDocument();
    // Declared, optional, and never supplied - which under a strict scheme is still not a finding.
    expect(render_.getByText("string?")).toBeInTheDocument();
    expect(render_.getByText("not supplied")).toBeInTheDocument();
  });

  it("draws plain rows for a section no scheme judges", async () => {
    // Most of a game tree: section schemes are an XRF convention, so nothing is undeclared against nothing.
    const render_: RenderResult = renderScheme({
      entry: ENTRY,
      fields: [fieldOf("cost", null, "100")],
      inheritedFrom: null,
      isDeclared: false,
      isStrict: false,
      scheme: null,
      section: "wpn_plain",
    });

    await waitFor(() => expect(render_.getByText("no scheme")).toBeInTheDocument());

    expect(render_.getByText("100")).toBeInTheDocument();
    expect(render_.queryByText("undeclared")).not.toBeInTheDocument();
  });

  it("marks a field the scheme never named, once a scheme is judging the section", async () => {
    const render_: RenderResult = renderScheme({
      entry: ENTRY,
      fields: [fieldOf("unexpected_field", null, "1")],
      inheritedFrom: null,
      isDeclared: true,
      isStrict: true,
      scheme: "$weapon",
      section: "wpn_broken",
    });

    await waitFor(() => expect(render_.getByText("undeclared")).toBeInTheDocument());
  });
});
