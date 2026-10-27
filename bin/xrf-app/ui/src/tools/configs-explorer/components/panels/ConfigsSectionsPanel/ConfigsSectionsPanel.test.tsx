import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { ConfigsDocument } from "@xrf/ipc/types/xrf-app-configs";
import { LtxFileStructure } from "@xrf/ipc/types/xrf-ltx-inspect";

import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { ConfigsDocumentService } from "@/tools/configs-explorer/services/configs-document";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/tools/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/tools/configs-explorer/services/configs-scheme";

import { ConfigsSectionsPanel } from "./ConfigsSectionsPanel";

const PATH: string = "valid_item_sections.ltx";

function getStructureOf(structure: Partial<LtxFileStructure>): LtxFileStructure {
  return {
    entryPoints: [PATH],
    includes: [],
    parseError: null,
    path: PATH,
    rootEntries: [],
    sections: [],
    ...structure,
  };
}

function renderPanel(structure: LtxFileStructure): { render: RenderResult; service: ConfigsDocumentService } {
  const container: Container = mockContainer([
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsResolvedService,
    ConfigsFindingsService,
    ConfigsSchemeService,
  ]);
  const service: ConfigsDocumentService = container.get(ConfigsDocumentService);

  service.selected = structure.path;
  service.document = service.document.asReady({
    findings: [],
    structure,
    text: { isNormalized: true, lines: [], path: structure.path },
  } as ConfigsDocument);

  return { service, render: renderWithProviders(<ConfigsSectionsPanel />, { container }) };
}

describe("ConfigsSectionsPanel", () => {
  it("lists what a list config declares instead of reporting that it declares no sections", async () => {
    // Anomaly's `valid_item_sections.ltx` and its kind: names outside any section, which is all the file holds. Saying
    // "no sections" about it describes the one thing it does not have and none of what it does.
    const {
      render: { getAllByRole, getByText, queryByText },
      service,
    } = renderPanel(
      getStructureOf({
        rootEntries: [
          { hasValue: false, line: 1, name: "af_ear" },
          { hasValue: false, line: 2, name: "ammo_9x18_ap_bad" },
          { hasValue: false, line: 3, name: "bandage" },
        ],
      })
    );

    const openAt = jest.spyOn(service, "openAt").mockResolvedValue(undefined);

    expect(getByText("Entries")).toBeInTheDocument();
    expect(queryByText("This document declares no sections.")).not.toBeInTheDocument();
    expect(getAllByRole("treeitem")).toHaveLength(3);

    // An entry belongs to no section, so it is opened at the line it is written on rather than revealed by name.
    await userEvent.click(getByText("ammo_9x18_ap_bad"));

    expect(openAt).toHaveBeenCalledWith(PATH, 2);
  });

  it("filters entries by what was typed", async () => {
    const {
      render: { getAllByRole, getByLabelText, getByText },
    } = renderPanel(
      getStructureOf({
        rootEntries: [
          { hasValue: false, line: 1, name: "af_ear" },
          { hasValue: false, line: 2, name: "ammo_9x18_ap_bad" },
        ],
      })
    );

    await userEvent.click(getByLabelText("Filter entries"));
    await userEvent.paste("ammo");

    expect(getAllByRole("treeitem")).toHaveLength(1);
    expect(getByText("ammo_9x18_ap_bad")).toBeInTheDocument();

    await userEvent.clear(getByLabelText("Filter entries"));
    await userEvent.click(getByLabelText("Filter entries"));
    await userEvent.paste("nothing");

    expect(getByText("No entry matches that.")).toBeInTheDocument();
  });

  it("lists the headers of an ordinary config and reveals one by name", async () => {
    const {
      render: { getByText },
      service,
    } = renderPanel(
      getStructureOf({
        path: "configs\\system.ltx",
        sections: [
          { line: 1, name: "wpn_base", operation: "", parents: [], scheme: null },
          { line: 5, name: "wpn_child", operation: "", parents: [], scheme: null },
        ],
      })
    );

    expect(getByText("Sections")).toBeInTheDocument();

    await userEvent.click(getByText("wpn_child"));

    // By name, because that is what both views address a section by and what the Scheme panel explains.
    expect(service.selectedSection).toBe("wpn_child");
  });

  it("says a config declares no sections when it declares nothing at all", () => {
    const {
      render: { getByText },
    } = renderPanel(getStructureOf({}));

    expect(getByText("This document declares no sections.")).toBeInTheDocument();
  });
});
