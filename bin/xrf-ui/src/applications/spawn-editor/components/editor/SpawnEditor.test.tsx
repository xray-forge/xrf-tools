import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";
import { Route, Routes } from "react-router-dom";

import { SpawnEditor } from "@/applications/spawn-editor/components/editor/SpawnEditor";
import { SpawnFile, SpawnHeaderChunk } from "@/core/bindings/types/xrf-db";
import { IEditorPanel, useEditorPanelsRegistry } from "@/core/shell/editor-shell";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { SpawnFileService } from "@/core/spawn/services";
import { mockSpawnFile } from "@/fixtures/mocks/spawn.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const SPAWN_PATH: string = "C:\\game\\gamedata\\spawns\\all.spawn";

/** Stands in for the window frame, which is what actually renders the published panels. */
function PublishedPanels(): ReactElement {
  const panels: ReadonlyArray<IEditorPanel> = useEditorPanelsRegistry();

  return <div data-testid={"published-tools"}>{panels.map((it: IEditorPanel) => it.label).join(",")}</div>;
}

/**
 * Configures mock responses for opening a spawn file.
 *
 * @param overrides - Spawn-file fields to replace in the default fixture.
 */
function mockOpenSpawn(overrides: Partial<SpawnFile> = {}): void {
  const file: SpawnFile = mockSpawnFile(overrides);

  setMockInvokeResponses({
    ["plugin:spawn|has_file"]: true,
    ["plugin:spawn|get_header"]: file.header,
    ["plugin:spawn|get_path"]: SPAWN_PATH,
    ["plugin:spawn|get_patrols"]: file.patrols,
    ["plugin:spawn|get_graphs"]: file.graphs,
    ["plugin:spawn|get_alife_spawns"]: file.alifeSpawn,
    ["plugin:spawn|get_artefact_spawns"]: file.artefactSpawn,
  });
}

describe("SpawnEditor", () => {
  beforeEach(() => {
    window.localStorage.clear();
    mockOpenSpawn();
  });

  function renderEditor(route: string = "/spawn-editor/header"): RenderResult {
    // The fixture renders panels published through the root shell service, including the chunk menu.
    return renderWithProviders(
      <>
        <Routes>
          <Route path={"/spawn-editor/*"} element={<SpawnEditor />} />
        </Routes>
        <ApplicationStatusBar />
        <PublishedPanels />
      </>,
      { route, bindings: [SpawnFileService] }
    );
  }

  it("titles itself from the route, not from a hardcoded string", async () => {
    const { findByText } = renderEditor();

    expect(await findByText("Spawn editor")).toBeInTheDocument();
  });

  it("offers every chunk section in the side menu", async () => {
    const { findByText } = renderEditor();

    for (const section of ["Header", "Alife", "Artefacts", "Patrols", "Graph"]) {
      expect(await findByText(section)).toBeInTheDocument();
    }
  });

  it("names the open file in the toolbar, which the backend has to report", async () => {
    const { findByText } = renderEditor();

    // The parsed file carries no path, so this only works because the session keeps it.
    expect(await findByText(SPAWN_PATH)).toBeInTheDocument();
  });

  it("keeps navigation in the menu and commands on the toolbar", async () => {
    const { findByRole, getByLabelText, getByRole, getByText, queryByText } = renderEditor();

    expect(await findByRole("button", { name: /Save/ })).toBeInTheDocument();
    expect(getByRole("button", { name: /Export/ })).toBeInTheDocument();
    expect(getByText("Alife")).toBeInTheDocument();

    expect(queryByText("Close")).not.toBeInTheDocument();
    expect(getByLabelText("Back to Spawn editor")).toBeInTheDocument();
  });

  it("publishes header counts to the status bar", async () => {
    const { findByText, getByText } = renderEditor();

    expect(await findByText("version 124")).toBeInTheDocument();
    expect(getByText("2 objects")).toBeInTheDocument();
    expect(getByText("1 levels")).toBeInTheDocument();
  });

  it("retries a failed chunk through the shared error state", async () => {
    const file: SpawnFile = mockSpawnFile();
    let isFailing: boolean = true;

    setMockInvokeResponses({
      ["plugin:spawn|has_file"]: true,
      ["plugin:spawn|get_path"]: SPAWN_PATH,
      ["plugin:spawn|get_header"]: () => {
        if (isFailing) {
          throw new Error("Chunk read failed");
        }

        return file.header;
      },
    });

    const { findByRole, getByRole, findByText, queryByRole } = renderEditor();

    expect(await findByRole("alert")).toHaveTextContent("Chunk read failed");

    isFailing = false;
    await userEvent.click(getByRole("button", { name: "Retry" }));

    expect(await findByText("version 124")).toBeInTheDocument();
    expect(queryByRole("alert")).not.toBeInTheDocument();
  });

  it("reflects the fixture rather than a fixed string", async () => {
    const header: SpawnHeaderChunk = {
      version: 118,
      guid: "guid",
      graphGuid: "graph-guid",
      objectsCount: 9001,
      levelsCount: 32,
    };

    mockOpenSpawn({ header });

    const { findByText, getByText } = renderEditor();

    expect(await findByText("9001 objects")).toBeInTheDocument();
    expect(getByText("version 118")).toBeInTheDocument();
    expect(getByText("32 levels")).toBeInTheDocument();
  });

  it("marks the active chunk section", async () => {
    const { findByText } = renderEditor();

    const header: HTMLElement = await findByText("Header");

    expect(header.closest("[role='button']")).toHaveClass("Mui-selected");
  });

  it("keeps the chunk section selected while a sub-table of it is showing", async () => {
    const { findByText } = renderEditor("/spawn-editor/graph/levels");

    const graph: HTMLElement = await findByText("Graph");

    // Selection used to be decided by the end of the path, so entering a sub-table deselected the
    // section it belongs to.
    expect(graph.closest("[role='button']")).toHaveClass("Mui-selected");
  });

  it("takes the visible sub-table from the route, so it can be linked to", async () => {
    // Asserted through the row count rather than a column header: the grid virtualises to nothing
    // without layout, which is why the shared table's own tests read its chrome too.
    const { findByText } = renderEditor("/spawn-editor/graph/levels");

    expect(await findByText("1 level(s)")).toBeInTheDocument();
  });

  it("defaults to the first sub-table when the route names none", async () => {
    const { findByText } = renderEditor("/spawn-editor/graph");

    expect(await findByText("1 header(s)")).toBeInTheDocument();
  });

  it("marks the sub-table tab the route selected", async () => {
    const { findByRole } = renderEditor("/spawn-editor/graph/vertices");

    expect(await findByRole("tab", { name: "Vertices", selected: true })).toBeInTheDocument();
  });

  it("publishes a details panel to the shell rather than rendering one itself", async () => {
    const { findByTestId } = renderEditor();

    // The stripe that renders these lives in the window frame, so what this can check is that the
    // editor declared it.
    expect(await findByTestId("published-tools")).toHaveTextContent("Row details");
  });
});
