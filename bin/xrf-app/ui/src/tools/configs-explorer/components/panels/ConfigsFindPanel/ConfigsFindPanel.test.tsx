import { describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { listMockInvokeCalls, mockIpcCommands } from "@xrf/ipc/testing";
import { ConfigsDeclarationSearch, ConfigsProjectDescriptor } from "@xrf/ipc/types/xrf-app-configs";
import { LtxDeclaration } from "@xrf/ipc/types/xrf-ltx-inspect";

import { mockSessionRead, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { ConfigsFindPanel } from "@/tools/configs-explorer/components/panels/ConfigsFindPanel";
import { mockRestoredConfigsProject } from "@/tools/configs-explorer/fixtures/configs-project";
import { ConfigsDocumentService } from "@/tools/configs-explorer/services/configs-document";
import { ConfigsFindService } from "@/tools/configs-explorer/services/configs-find";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/tools/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/tools/configs-explorer/services/configs-scheme";

const BASE: string = "configs\\items\\w_ak74.ltx";
const PATCH: string = "configs\\mod_system_xxx.ltx";

/** The weapon's declaration, read out of an archive. */
const DECLARED: LtxDeclaration = {
  isPatch: false,
  isPhysical: false,
  kind: "section",
  line: 1,
  parents: ["wpn_base"],
  path: BASE,
  section: "wpn_ak74",
  sectionOperation: "",
  source: "database\\gamedata.db (archive)",
};

/** A patch file's override of it, loose beside the base tree. */
const PATCHED: LtxDeclaration = {
  isPatch: true,
  isPhysical: true,
  kind: "section",
  line: 3,
  parents: [],
  path: PATCH,
  section: "wpn_ak74",
  sectionOperation: "!",
  source: "gamedata",
};

/** What the backend answers, holding `declarations` of `total`. */
function toFound(declarations: Array<LtxDeclaration>, total: number = declarations.length): ConfigsDeclarationSearch {
  return {
    search: {
      declarations,
      query: { field: null, isSectionCaseFolded: true, section: "wpn_ak74", value: null },
      unreadable: [],
    },
    total,
  };
}

/** The panel over an open project whose search answers `found`. */
async function renderFind(found: ConfigsDeclarationSearch): Promise<{
  render: RenderResult;
  documentService: ConfigsDocumentService;
}> {
  const container: Container = mockContainer([
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsFindingsService,
    ConfigsResolvedService,
    ConfigsSchemeService,
    ConfigsFindService,
  ]);

  await mockRestoredConfigsProject(container.get(ConfigsProjectService));

  mockIpcCommands({
    [EConfigsCommand.RESTORE_PROJECT]: mockSessionSnapshot({} as ConfigsProjectDescriptor, "session-1"),
    [EConfigsCommand.FIND_DECLARATIONS]: mockSessionRead(found),
    // What opening a row reads: the config it names.
    [EConfigsCommand.READ_DOCUMENT]: mockSessionRead({
      findings: [],
      structure: { entryPoints: [], includes: [], parseError: null, path: PATCH, rootEntries: [], sections: [] },
      text: { isNormalized: true, lines: ["", "", "![wpn_ak74]"], path: PATCH },
    }),
  });

  return {
    documentService: container.get(ConfigsDocumentService),
    render: renderWithProviders(<ConfigsFindPanel />, { container }),
  };
}

describe("ConfigsFindPanel", () => {
  it("searches the project for what was typed and says how each config writes it", async () => {
    const { render } = await renderFind(toFound([DECLARED, PATCHED]));

    await userEvent.type(render.getByLabelText("Section"), "wpn_ak74{Enter}");

    await waitFor(() => expect(render.getAllByTestId("configs-find-row")).toHaveLength(2));

    expect(listMockInvokeCalls(EConfigsCommand.FIND_DECLARATIONS).at(-1)).toEqual({
      request: { field: null, section: "wpn_ak74", sessionId: expect.any(String), value: null },
    });

    // As written: the base header with its parent, the override with its prefix, and which one is archived.
    expect(render.getByText("[wpn_ak74]:wpn_base")).toBeInTheDocument();
    expect(render.getByText("![wpn_ak74]")).toBeInTheDocument();
    expect(render.getByText("declaration")).toBeInTheDocument();
    expect(render.getByText("patch")).toBeInTheDocument();
    expect(render.getAllByText("archived")).toHaveLength(1);
  });

  it("finds the section selected in the document without typing it", async () => {
    const { render, documentService } = await renderFind(toFound([DECLARED]));

    documentService.selectSection("wpn_ak74");

    await userEvent.click(await render.findByRole("button", { name: "Find the selected section" }));

    await waitFor(() => expect(render.getAllByTestId("configs-find-row")).toHaveLength(1));
    expect(render.getByLabelText("Section")).toHaveValue("wpn_ak74");
  });

  it("opens the config a row names at the line it is written on", async () => {
    const { render, documentService } = await renderFind(toFound([PATCHED]));

    await userEvent.type(render.getByLabelText("Section"), "wpn_ak74{Enter}");
    await userEvent.click(await render.findByText("mod_system_xxx.ltx:3"));

    await waitFor(() => expect(documentService.selected).toBe(PATCH));
    expect(documentService.revealed).toEqual({ kind: "line", line: 3 });
  });

  it("asks the backend nothing when nothing was typed, which it would refuse", async () => {
    const { render } = await renderFind(toFound([DECLARED]));

    await userEvent.click(render.getByRole("button", { name: "Find" }));

    expect(listMockInvokeCalls(EConfigsCommand.FIND_DECLARATIONS)).toHaveLength(0);
    expect(render.getByText(/Name a section, a field or a value/)).toBeInTheDocument();
  });

  it("says when the answer holds only the first of what matched", async () => {
    const { render } = await renderFind(toFound([DECLARED], 1200));

    await userEvent.type(render.getByLabelText("Field"), "cost{Enter}");

    await waitFor(() => expect(render.getByTestId("configs-find-truncated")).toHaveTextContent(/first 1 of 1,200/));
  });
});
