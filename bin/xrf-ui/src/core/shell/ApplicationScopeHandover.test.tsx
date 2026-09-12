import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect } from "react";
import { NavigateFunction, Route, Routes, useLocation, useNavigate } from "react-router-dom";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ApplicationShell } from "@/core/shell/ApplicationShell";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

/** Injects the way the archives menu does, which is what makes the handover observable. */
function ArchivesScopedPanel(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  return <div>{`archives panel ${archivesService.project.isReady}`}</div>;
}

/** Publishes a left panel bound to its own application's container, as the archives editor does. */
function ArchivesLikeEditor(): ReactElement {
  const navigate: NavigateFunction = useNavigate();

  useEditorStatus(["Archive project status"]);

  useEditorPanels(
    () => [
      {
        icon: <span>a</span>,
        id: "archives-menu",
        isOpenByDefault: true,
        label: "Archives",
        render: () => <ArchivesScopedPanel />,
        side: "left",
      },
    ],
    []
  );

  return <button onClick={() => navigate("/spawn-editor", { replace: true })}>leave</button>;
}

function ArchivesNavigationProbe({ onObserve }: { onObserve: (service: ArchivesService) => void }): ReactElement {
  const service: ArchivesService = useInjection(ArchivesService);
  const { pathname } = useLocation();
  const navigate = useNavigate();

  useEffect(() => onObserve(service), [onObserve, pathname, service]);

  return <button onClick={() => navigate("/archives-explorer/entry")}>Inspect entry</button>;
}

describe("panel handover between applications", () => {
  beforeEach(() => {
    window.localStorage.clear();
    setMockInvokeResponses({});
  });

  it("keeps the loaded services during navigation within an application", async () => {
    const onObserve = jest.fn<(service: ArchivesService) => void>();
    const view = await act(async () =>
      renderWithProviders(
        <ApplicationShell>
          <ArchivesNavigationProbe onObserve={onObserve} />
        </ApplicationShell>,
        { route: "/archives-explorer", hasShell: true }
      )
    );

    expect(onObserve).toHaveBeenCalledTimes(1);

    await userEvent.click(view.getByRole("button", { name: "Inspect entry" }));

    expect(onObserve).toHaveBeenCalledTimes(2);
    expect(onObserve.mock.calls[1][0]).toBe(onObserve.mock.calls[0][0]);
  });

  it("stops rendering an application's panels the moment its container goes away", async () => {
    // The registry is cleared by an effect, but the container is swapped during render. For the commit
    // in between, the frame held the outgoing application's panels and the incoming one's container -
    // so a panel that injects asked a container that never bound its service.
    const { getByText, findByText, queryByText } = await act(async () =>
      renderWithProviders(
        <ApplicationShell>
          <Routes>
            <Route path={"/archives-explorer/*"} element={<ArchivesLikeEditor />} />
            <Route path={"/spawn-editor/*"} element={<div>spawn editor</div>} />
          </Routes>
        </ApplicationShell>,
        { route: "/archives-explorer", hasShell: true }
      )
    );

    expect(await findByText(/archives panel/)).toBeInTheDocument();
    expect(getByText("Archive project status")).toBeInTheDocument();

    await act(async () => fireEvent.click(getByText("leave")));

    expect(await findByText("spawn editor")).toBeInTheDocument();
    expect(queryByText(/archives panel/)).not.toBeInTheDocument();
    expect(queryByText("Archive project status")).not.toBeInTheDocument();
    expect(getByText("Ready")).toBeInTheDocument();
  });
});
