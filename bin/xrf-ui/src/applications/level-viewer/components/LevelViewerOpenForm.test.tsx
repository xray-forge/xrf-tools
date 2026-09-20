import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { LevelViewerOpenForm } from "@/applications/level-viewer/components/LevelViewerOpenForm";
import { AssetService } from "@/core/assets/services";
import { LevelEntry } from "@/core/ipc/types/xrf-app";
import { LevelListService, LevelLoadService } from "@/core/level/services";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

const INSTALLATION: string = "C:\\game";
const OTHER_INSTALLATION: string = "C:\\anomaly";

function mockEntry(name: string): LevelEntry {
  return { hasGeometry: true, logicalPath: `levels\\${name}`, name };
}

function renderForm(): RenderResult {
  const container: Container = mockContainer([AssetService, LevelListService, LevelLoadService]);

  return renderWithProviders(<LevelViewerOpenForm />, { route: "/level-viewer", container });
}

function setRoot(form: RenderResult, path: string): void {
  fireEvent.change(form.getByRole("textbox", { name: "Game root" }), { target: { value: path } });
}

describe("LevelViewerOpenForm", () => {
  beforeEach(() => {
    window.localStorage.clear();
    resetMockInvoke();

    setMockInvokeResponses({ ["plugin:levels|list_levels"]: [mockEntry("zaton"), mockEntry("jupiter")] });
  });

  it("lists what the root holds", async () => {
    const form: RenderResult = renderForm();

    setRoot(form, INSTALLATION);
    await userEvent.click(form.getByRole("button", { name: "List levels" }));

    expect(await form.findByRole("option", { name: "zaton" })).toBeInTheDocument();
    expect(form.getByRole("option", { name: "jupiter" })).toBeInTheDocument();
  });

  // Which level is opened is the one decision this form exists to take, and a preselected first entry is one that
  // gets submitted without being read.
  it("opens nothing until a level has been chosen", async () => {
    const form: RenderResult = renderForm();

    setRoot(form, INSTALLATION);
    await userEvent.click(form.getByRole("button", { name: "List levels" }));

    expect(await form.findByRole("button", { name: "Open" })).toBeDisabled();

    await userEvent.click(form.getByRole("option", { name: "zaton" }));

    expect(form.getByRole("button", { name: "Open" })).toBeEnabled();
  });

  // A listing belongs to the root it was made from: a level of that name under another root is another file.
  it("forgets the levels when the root changes", async () => {
    const form: RenderResult = renderForm();

    setRoot(form, INSTALLATION);
    await userEvent.click(form.getByRole("button", { name: "List levels" }));
    await userEvent.click(await form.findByRole("option", { name: "zaton" }));

    setRoot(form, OTHER_INSTALLATION);

    expect(form.queryByRole("option", { name: "zaton" })).not.toBeInTheDocument();
    expect(form.getByRole("button", { name: "List levels" })).toBeInTheDocument();
  });

  it("asks for the choice again once the new root has been listed", async () => {
    const form: RenderResult = renderForm();

    setRoot(form, INSTALLATION);
    await userEvent.click(form.getByRole("button", { name: "List levels" }));
    await userEvent.click(await form.findByRole("option", { name: "zaton" }));

    setRoot(form, OTHER_INSTALLATION);
    await userEvent.click(form.getByRole("button", { name: "List levels" }));

    expect(await form.findByRole("button", { name: "Open" })).toBeDisabled();
  });

  it("leaves a failed listing on its own button to retry", async () => {
    setMockInvokeResponses({
      ["plugin:levels|list_levels"]: () => {
        throw new Error("no roots are mounted");
      },
    });

    const form: RenderResult = renderForm();

    setRoot(form, INSTALLATION);
    await userEvent.click(form.getByRole("button", { name: "List levels" }));

    expect(await form.findByText("no roots are mounted")).toBeInTheDocument();
    expect(form.getByRole("button", { name: "List levels" })).toBeInTheDocument();
  });
});
