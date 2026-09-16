import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ArchivesExplorerApplication } from "@/applications/archives-explorer/ArchivesExplorerApplication";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import { ArchiveWorldEntry } from "@/core/ipc/types/xrf-app";
import { ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";
import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import {
  mockArchivedContainer,
  mockArchiveFileDescriptor,
  mockArchiveShadowedCopy,
  mockArchivesVolumes,
  mockArchivesWorldSubject,
  mockArchiveWorldEntry,
  mockArchiveWorldStatistics,
  mockPathCollision,
} from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const BASE: string = "C:\\game\\db\\base.db0";
const PATCH: string = "C:\\game\\db\\patch.db1";
const TEXTURES: string = "C:\\game\\db\\textures.db0";

function mockStatistics(): ArchiveStatistics {
  return mockArchiveWorldStatistics({
    origins: {
      archived: { files: 2, sizeReal: 6144 },
      hidden: { files: 2, sizeReal: 1536 },
      loose: { files: 0, sizeReal: 0 },
      sources: [
        { hides: { files: 0, sizeReal: 0 }, isLoose: false, source: PATCH, wins: { files: 2, sizeReal: 6144 } },
        { hides: { files: 1, sizeReal: 1024 }, isLoose: false, source: BASE, wins: { files: 0, sizeReal: 0 } },
        { hides: { files: 1, sizeReal: 512 }, isLoose: false, source: TEXTURES, wins: { files: 0, sizeReal: 0 } },
      ],
    },
  });
}

const PATCHED: ArchiveWorldEntry = mockArchiveWorldEntry({
  container: mockArchivedContainer(PATCH),
  name: "configs\\system.ltx",
  shadowed: [mockArchiveShadowedCopy(mockArchivedContainer(BASE), 1024)],
  sizeReal: 2048,
});

const TEXTURE: ArchiveWorldEntry = mockArchiveWorldEntry({
  container: mockArchivedContainer(PATCH),
  name: "textures\\wpn\\ak74.dds",
  shadowed: [mockArchiveShadowedCopy(mockArchivedContainer(TEXTURES), 512)],
  sizeReal: 4096,
});

async function renderExplorer() {
  return act(async () =>
    renderWithProviders(
      <ApplicationShellFrame>
        <ArchivesExplorerApplication />
      </ApplicationShellFrame>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService], hasShell: true }
    )
  );
}

describe("archive overrides dialog", () => {
  beforeEach(() => {
    window.localStorage.clear();

    setMockInvokeResponses({
      ["plugin:archives|describe_statistics"]: mockStatistics(),
      ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesWorldSubject([PATCHED, TEXTURE])),
      ["plugin:archives|list_overrides"]: { overridden: [PATCHED, TEXTURE], unreachable: [] },
      ["plugin:archives|read_file"]: { name: PATCHED.name, content: "[system]", size: PATCHED.sizeReal },
    });
  });

  it("draws every copy of a contested path, winner first and marked", async () => {
    const { findByLabelText, findAllByTestId, findAllByText, findByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Overrides"));

    expect(await findByText("configs\\system.ltx")).toBeInTheDocument();

    // The winner is drawn rather than implied: a stack showing only what lost leaves the losers with nothing named
    // to have lost to, which is the whole mechanism this dialog exists to show.
    // The patch volume wins both paths, so it is drawn once per contest as well as once in the breakdown above.
    expect((await findAllByText(PATCH)).length).toBeGreaterThan(1);
    expect((await findAllByText(BASE)).length).toBeGreaterThan(0);
    expect((await findAllByTestId("archive-override-row")).length).toBeGreaterThan(2);
  });

  it("scopes the listing to a source, and clears it when the same source is picked again", async () => {
    const { findAllByTestId, findByLabelText, findByText, queryByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Overrides"));

    // The breakdown row, not a copy row: the aggregate above is what scopes the listing below it.
    const source: HTMLElement = (await findAllByTestId("stat-breakdown-row")).find((row: HTMLElement) =>
      row.textContent?.includes(TEXTURES)
    ) as HTMLElement;

    await userEvent.click(source);

    // Matching any copy of the path, not only the winner: asking what a volume buries is asking about the copies it
    // buried, which are never the ones that won.
    expect(await findByText("textures\\wpn\\ak74.dds")).toBeInTheDocument();
    expect(queryByText("configs\\system.ltx")).toBeNull();

    await userEvent.click(source);

    expect(await findByText("configs\\system.ltx")).toBeInTheDocument();
  });

  it("narrows the listing by engine path", async () => {
    const { findByLabelText, findByText, queryByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Overrides"));
    await userEvent.type(await findByLabelText("Filter overridden paths"), "ak74");

    expect(await findByText("textures\\wpn\\ak74.dds")).toBeInTheDocument();
    expect(queryByText("configs\\system.ltx")).toBeNull();
  });

  it("opens the file a row names and closes itself", async () => {
    const { findByLabelText, findByText, queryByRole } = await renderExplorer();

    await userEvent.click(await findByLabelText("Overrides"));
    await userEvent.click(await findByText("configs\\system.ltx"));

    // Finding a contested path and then looking at it is one motion, so the dialog gets out of the way rather than
    // covering the tree it just navigated.
    expect(queryByRole("dialog", { name: "Overrides" })).toBeNull();
  });

  it("proves a clean subject is clean rather than saying nothing", async () => {
    setMockInvokeResponses({
      ["plugin:archives|describe_statistics"]: mockStatistics(),
      ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesWorldSubject([])),
      ["plugin:archives|list_overrides"]: { overridden: [], unreachable: [] },
    });

    const { findByLabelText, findByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Overrides"));

    expect(await findByText(/No engine path here is held more than once/)).toBeInTheDocument();
    expect(await findByText(/Every source here reaches each of its own entries/)).toBeInTheDocument();
  });

  it("names both spellings of a copy nothing reaches, and marks which one loads", async () => {
    setMockInvokeResponses({
      ["plugin:archives|describe_statistics"]: mockStatistics(),
      ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesWorldSubject([])),
      ["plugin:archives|list_overrides"]: { overridden: [], unreachable: [mockPathCollision()] },
    });

    const { findByLabelText, findByText, queryByRole } = await renderExplorer();

    await userEvent.click(await findByLabelText("Overrides"));

    expect(await findByText("C:/game/database/configs.db0::textures/a.dds")).toBeInTheDocument();
    expect(await findByText("C:/game/database/patch.db0::Textures/A.DDS")).toBeInTheDocument();
    expect(await findByText("Unreachable")).toBeInTheDocument();

    await userEvent.click(await findByText("textures\\a.dds"));

    expect(queryByRole("dialog", { name: "Overrides" })).not.toBeNull();
  });

  it("opens the file a row names on a volume set, which spells its entries as authored", async () => {
    const AUTHORED: string = "Configs\\System.LTX";

    setMockInvokeResponses({
      ["plugin:archives|describe_statistics"]: mockStatistics(),
      ["plugin:archives|get_subject"]: mockSessionResponse(
        mockArchivesVolumes([mockArchiveFileDescriptor({ name: AUTHORED })])
      ),
      ["plugin:archives|list_overrides"]: {
        overridden: [
          mockArchiveWorldEntry({
            container: mockArchivedContainer(PATCH),
            name: "configs\\system.ltx",
            shadowed: [mockArchiveShadowedCopy(mockArchivedContainer(BASE), 1024)],
            sizeReal: 2048,
          }),
        ],
        unreachable: [],
      },
      ["plugin:archives|read_file"]: { content: "[system]", name: AUTHORED, size: 2048 },
    });

    const { findByLabelText, findByText, queryByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Overrides"));
    await userEvent.click(await findByText("configs\\system.ltx"));

    expect(await findByText(AUTHORED)).toBeInTheDocument();
    expect(queryByText("Select a file to preview")).toBeNull();
  });
});
