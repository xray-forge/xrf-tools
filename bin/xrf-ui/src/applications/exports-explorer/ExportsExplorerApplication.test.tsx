import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { Route, Routes } from "react-router-dom";

import { ExportsExplorerApplication } from "@/applications/exports-explorer/ExportsExplorerApplication";
import { ApplicationShell } from "@/core/shell/ApplicationShell";
import { mockExportsProject } from "@/fixtures/mocks/project.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ExportsExplorerApplication", () => {
  beforeEach(() => {
    window.localStorage.clear();
    // The picker remembers its own path: nothing configured describes a TypeScript source tree, so exports asks.
    window.localStorage.setItem("xrf.form.exports-explorer.project", "C:\\projects\\active-xrf");

    setMockInvokeResponses({
      ["plugin:exports|get_project"]: null,
      ["plugin:exports|open_project"]: mockExportsProject({ root: "C:\\projects\\active-xrf" }),
    });
  });

  /**
   * Renders the exports application through its shell.
   *
   * @param route - Initial application route.
   * @returns Testing Library render result for the application shell.
   */
  function renderApplication(route: string) {
    return renderWithProviders(
      <ApplicationShell>
        <Routes>
          <Route path={"/exports-explorer/*"} element={<ExportsExplorerApplication />} />
          <Route path={"/"} element={<div>Application home</div>} />
        </Routes>
      </ApplicationShell>,
      { route }
    );
  }

  it("lands on its own picker, with no list of one thing in between", async () => {
    // The route used to open a landing pane holding a single card called "Open". Flattening deleted
    // that pane: the application is the thing home links to, so it opens what it is for.
    const { findByDisplayValue, findByText, queryByText } = renderApplication("/exports-explorer");

    expect(await findByText("Open script exports")).toBeInTheDocument();
    expect(await findByDisplayValue("C:\\projects\\active-xrf")).toBeInTheDocument();
    expect(queryByText("Open")).not.toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:exports|open_project", expect.anything());
  });

  it("resolves the services its descriptor declares, with nothing bound above the shell", async () => {
    // `ExportsService` is bound by the frame out of `EXPORTS_EXPLORER_APPLICATION.bindings`. Only the root
    // services are provided here, so if that wiring broke this would throw rather than render.
    const { findByRole } = renderApplication("/exports-explorer");

    await userEvent.click(await findByRole("button", { name: "Open exports" }));

    expect(mockInvoke).toHaveBeenCalledWith("plugin:exports|open_project", {
      projectPath: "C:\\projects\\active-xrf",
    });
  });
});
