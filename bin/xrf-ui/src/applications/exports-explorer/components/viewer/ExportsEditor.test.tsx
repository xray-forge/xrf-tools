import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { fireEvent, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ExportsExplorerApplication } from "@/applications/exports-explorer/ExportsExplorerApplication";
import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { ExportsProject } from "@/core/bindings/types/xrf-export";
import { TCallableExportDescriptor, TValueExportDescriptor } from "@/core/exports";
import { ProjectService } from "@/core/settings/services/project";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Logger } from "@/lib/logging";

const PLAY_SOUND: TCallableExportDescriptor = {
  kind: "callable",
  name: "xr_effects.play_sound",
  description: "Plays an actor sound.",
  parameters: [
    { name: "actor", typing: "game_object", description: "Target stalker.", isOptional: false },
    { name: "volume", typing: "number", description: "Optional volume.", isOptional: true },
  ],
  returns: { typing: "boolean", description: "Whether playback started." },
  source: { path: "src/engine/declarations/effects/sound.ts", line: 18, column: 3, endLine: 21 },
};

const SETTINGS: TValueExportDescriptor = {
  kind: "value",
  name: "settings",
  description: "Shared configuration.",
  typing: "Record<string, boolean>",
  source: { path: "src/engine/declarations/settings.ts", line: 4, column: 1, endLine: 7 },
};

const DIALOG: TCallableExportDescriptor = {
  kind: "callable",
  name: "dialogs_zaton.quest.answer",
  description: null,
  parameters: [],
  returns: { typing: "void", description: null },
  source: { path: "src/engine/declarations/dialogs/answer.ts", line: 9, column: 2, endLine: 12 },
};

const PROJECT: ExportsProject = {
  root: "C:\\projects\\xrf",
  declarations: [DIALOG, SETTINGS, PLAY_SOUND],
};

describe("opened exports editor", () => {
  beforeEach(() => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: PROJECT,
      ["plugin:exports|open_project"]: PROJECT,
      ["plugin:exports|close_project"]: undefined,
    });
  });

  function renderEditor() {
    return renderWithProviders(
      <>
        <ExportsExplorerApplication />
        <ApplicationStatusBar />
      </>,
      { route: "/exports-explorer", bindings: [ProjectService, ExportsService] }
    );
  }

  it("shows project context, aggregate status, collapsed groups, and a guided empty state", async () => {
    const { findByText, getByText, queryByText } = renderEditor();

    expect(await findByText("Select an export to inspect")).toBeInTheDocument();
    expect(getByText("C:\\projects\\xrf")).toBeInTheDocument();
    expect(getByText("3 exports")).toBeInTheDocument();
    expect(getByText("3 groups")).toBeInTheDocument();
    expect(getByText("~ (1)")).toBeInTheDocument();
    expect(getByText("dialogs_zaton (1)")).toBeInTheDocument();
    expect(getByText("xr_effects (1)")).toBeInTheDocument();
    expect(queryByText("xr_effects.play_sound")).not.toBeInTheDocument();
  });

  it("expands namespaces without selecting and renders the complete selected declaration", async () => {
    const { findByRole, findByText, getByText } = renderEditor();

    await userEvent.dblClick(await findByText("xr_effects (1)"));

    expect(getByText("Select an export to inspect")).toBeInTheDocument();

    await userEvent.dblClick(await findByText("xr_effects.play_sound"));

    expect(getByText("Callable")).toBeInTheDocument();
    expect(getByText("xr_effects.play_sound(actor: game_object, volume?: number): boolean")).toBeInTheDocument();
    expect(getByText("Plays an actor sound.")).toBeInTheDocument();
    expect(getByText("Target stalker.")).toBeInTheDocument();
    expect(getByText("Optional volume.")).toBeInTheDocument();
    expect(getByText("Whether playback started.")).toBeInTheDocument();
    expect(getByText("src/engine/declarations/effects/sound.ts:18:3")).toBeInTheDocument();
    expect(await findByRole("table", { name: "Export parameters" })).toBeInTheDocument();
  });

  it("searches all groups by documentation and restores collapsed state after clearing", async () => {
    const { findByRole, findByText, getByLabelText, queryByText } = renderEditor();
    const search: HTMLElement = await findByRole("textbox", { name: "Filter exports" });

    fireEvent.change(search, { target: { value: "TARGET STALKER" } });

    expect(await findByText("play_sound")).toBeInTheDocument();
    expect(await findByText("xr_effects")).toBeInTheDocument();
    expect(queryByText("settings")).not.toBeInTheDocument();

    await userEvent.click(getByLabelText("Clear filter"));

    await waitFor(() => expect(queryByText("play_sound")).not.toBeInTheDocument());
    expect(search).toHaveValue("");
  });

  it("refreshes explicitly and preserves a declaration that still exists", async () => {
    const refreshed: ExportsProject = {
      ...PROJECT,
      declarations: [{ ...PLAY_SOUND, description: "Updated sound documentation." }, SETTINGS, DIALOG],
    };

    setMockInvokeResponses({
      ["plugin:exports|get_project"]: PROJECT,
      ["plugin:exports|open_project"]: refreshed,
    });

    const { findByLabelText, findByText } = renderEditor();

    await userEvent.dblClick(await findByText("xr_effects (1)"));
    await userEvent.dblClick(await findByText("xr_effects.play_sound"));
    await userEvent.click(await findByLabelText("Refresh exports"));

    expect(await findByText("Updated sound documentation.")).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith("plugin:exports|open_project", {
      projectPath: PROJECT.root,
    });
  });

  it("keeps the selected snapshot and reports a refresh failure", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: PROJECT,
      ["plugin:exports|open_project"]: () => {
        throw new Error("invalid declaration");
      },
    });

    const { findByLabelText, findByText, getByText } = renderEditor();

    await userEvent.dblClick(await findByText("xr_effects (1)"));
    await userEvent.dblClick(await findByText("xr_effects.play_sound"));
    await userEvent.click(await findByLabelText("Refresh exports"));

    expect(await findByText("Could not refresh exports: invalid declaration")).toBeInTheDocument();
    expect(getByText("Plays an actor sound.")).toBeInTheDocument();
  });

  it("closes into its own picker rather than navigating away", async () => {
    // Closing used to leave for the exports landing pane, which held a single card. The application
    // draws its own picker whenever nothing is open, so there is nowhere else to go.
    const { findByLabelText, findByText } = renderWithProviders(<ExportsExplorerApplication />, {
      route: "/exports-explorer",
      bindings: [ProjectService, ExportsService],
    });

    await userEvent.click(await findByLabelText("Back to Exports explorer"));

    expect(await findByText("Open script exports")).toBeInTheDocument();
    await waitFor(() => {
      const closeCalls = mockInvoke.mock.calls.filter(([command]) => command === "plugin:exports|close_project");

      expect(closeCalls.length).toBeGreaterThanOrEqual(2);
    });
  });

  it("stays open and reports a close failure", async () => {
    const releaseError = jest.spyOn(Logger, "error").mockImplementation(() => undefined);

    setMockInvokeResponses({
      ["plugin:exports|get_project"]: PROJECT,
      ["plugin:exports|close_project"]: () => {
        throw new Error("project is busy");
      },
    });

    const { findByLabelText, findByText, getByText, unmount } = renderEditor();

    await userEvent.click(await findByLabelText("Back to Exports explorer"));

    expect(await findByText("Could not close exports: project is busy")).toBeInTheDocument();
    // Still the viewer, not the picker: a failed close leaves the project loaded.
    expect(getByText(PROJECT.root)).toBeInTheDocument();

    unmount();
    await waitFor(() => expect(releaseError).toHaveBeenCalled());
    releaseError.mockRestore();
  });
});

describe("empty exports editor", () => {
  it("keeps an empty project open", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: { root: PROJECT.root, declarations: [] },
    });

    const { findAllByText, findByText } = renderWithProviders(<ExportsExplorerApplication />, {
      route: "/exports-explorer",
      bindings: [ProjectService, ExportsService],
    });

    expect((await findAllByText("No externs found")).length).toBeGreaterThan(0);
    expect(await findByText(PROJECT.root)).toBeInTheDocument();
  });
});
