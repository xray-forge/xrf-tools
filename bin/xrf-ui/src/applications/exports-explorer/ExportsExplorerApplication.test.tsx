import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Route, Routes } from "react-router-dom";

import { ExportsExplorerApplication } from "@/applications/exports-explorer/ExportsExplorerApplication";
import { ApplicationShell } from "@/core/shell/ApplicationShell";
import { mockExportsProject } from "@/fixtures/mocks/project.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ExportsExplorerApplication", () => {
  beforeEach(() => {
    window.localStorage.clear();
    // The picker remembers its own path: nothing configured describes a TypeScript source tree, so exports asks.
    window.localStorage.setItem("xrf.form.exports-explorer.project", "C:\\projects\\active-xrf");

    setMockInvokeResponses({
      ["plugin:exports|get_project"]: mockSessionResponse(null),
      ["plugin:exports|open_project"]: mockSessionResponse(mockExportsProject({ root: "C:\\projects\\active-xrf" })),
    });
  });

  /**
   * Renders the exports application through its shell.
   *
   * @param route - Initial application route.
   * @returns Testing Library render result for the application shell.
   */
  async function renderApplication(route: string): Promise<RenderResult> {
    return act(async () =>
      renderWithProviders(
        <ApplicationShell>
          <Routes>
            <Route path={"/exports-explorer/*"} element={<ExportsExplorerApplication />} />
            <Route path={"/"} element={<div>Application home</div>} />
          </Routes>
        </ApplicationShell>,
        { route, hasShell: true }
      )
    );
  }

  it("lands on its own picker, with no list of one thing in between", async () => {
    // The route used to open a landing pane holding a single card called "Open". Flattening deleted
    // that pane: the application is the thing home links to, so it opens what it is for.
    const { findByDisplayValue, findByText, queryByText } = await renderApplication("/exports-explorer");

    expect(await findByText("Open script exports")).toBeInTheDocument();
    expect(await findByDisplayValue("C:\\projects\\active-xrf")).toBeInTheDocument();
    expect(queryByText("Open")).not.toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:exports|open_project", expect.anything());
  });

  it("resolves the services its descriptor declares, with nothing bound above the shell", async () => {
    // `ExportsService` is bound by the frame from the deferred runtime. Only the root
    // services are provided here, so if that wiring broke this would throw rather than render.
    const { findByRole } = await renderApplication("/exports-explorer");

    await userEvent.click(await findByRole("button", { name: "Open exports" }));

    expect(mockInvoke).toHaveBeenCalledWith("plugin:exports|open_project", {
      sessionId: expect.any(String),
      projectPath: "C:\\projects\\active-xrf",
    });
  });
});
