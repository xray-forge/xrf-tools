import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { useInjection } from "@wirestate/react";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { ReactElement } from "react";
import { NavigateFunction, Route, Routes, useNavigate } from "react-router-dom";

import { ApplicationShell } from "@/core/shell/ApplicationShell";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

function ArchivesScopedPanel(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  return <div>{`archives panel ${archivesService.subject.isRestored}`}</div>;
}

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

describe("panel handover between tools", () => {
  beforeEach(() => {
    clearLocalStorage();
    mockIpcCommands({});
  });

  it("stops rendering a tool's panels the moment its container goes away", async () => {
    // The registry is cleared by an effect, but the container is swapped during render. For the commit
    // in between, the frame held the outgoing tool's panels and the incoming one's container -
    // so a panel that injects asked a container that never bound its service.
    const { getByTestId, getByText, findByText, queryByText } = await act(async () =>
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
    expect(getByTestId("application-status-bar")).toBeEmptyDOMElement();
  });
});
