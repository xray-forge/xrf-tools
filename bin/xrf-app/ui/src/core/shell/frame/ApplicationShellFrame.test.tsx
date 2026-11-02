import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { CommandBus, Container, QueryBus } from "@wirestate/core";
import { EJobsCommand } from "@xrf/ipc/commands/jobs";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { ReactElement, ReactNode } from "react";

import { NotificationsService } from "@/core/notifications/services";
import { useShellPanels } from "@/core/shell/contributions";
import { ApplicationShellFrame } from "@/core/shell/frame/ApplicationShellFrame";
import {
  IPanelSetActiveCommand,
  IPanelSideCommand,
  PANEL_ACTIVE_QUERY,
  PANEL_SET_ACTIVE_COMMAND,
} from "@/core/shell/panels/panel-messages";
import { clearLocalStorage, setLocalStorageValue } from "@/core/storage";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

/** Stands in for an editor whose browse panel starts closed, which is the case the panel command exists for. */
function EditorWithClosedLeftPanel(): ReactElement {
  useShellPanels(
    () => [
      {
        icon: <span>t</span>,
        id: "tree",
        isOpenByDefault: false,
        label: "Tree",
        render: () => <div>tree panel</div>,
        side: "left",
      },
    ],
    []
  );

  return <div>closed editor</div>;
}

function withoutActEnvironment(run: () => void): void {
  const scope: Record<string, unknown> = globalThis as unknown as Record<string, unknown>;
  const previous: unknown = scope.IS_REACT_ACT_ENVIRONMENT;

  scope.IS_REACT_ACT_ENVIRONMENT = false;

  try {
    run();
  } finally {
    scope.IS_REACT_ACT_ENVIRONMENT = previous;
  }
}

function renderFrame(children: ReactNode): RenderResult {
  return renderWithProviders(<ApplicationShellFrame>{children}</ApplicationShellFrame>, {
    bindings: [NotificationsService],
  });
}

describe("ApplicationShellFrame", () => {
  beforeEach(() => {
    // The frame remembers which panel was open, so a leftover choice would decide the next test.
    clearLocalStorage();
  });

  it("opens a panel synchronously, so what a caller wants inside it is mounted when the dispatch returns", () => {
    const container: Container = mockContainer();
    const { getByText, queryByText } = renderWithProviders(
      <ApplicationShellFrame>
        <EditorWithClosedLeftPanel />
      </ApplicationShellFrame>,
      { container, hasShell: true, route: "/" }
    );

    expect(queryByText("tree panel")).toBeNull();

    withoutActEnvironment(() =>
      container
        .get(CommandBus)
        .execute<void, IPanelSetActiveCommand>(PANEL_SET_ACTIVE_COMMAND, { panelId: "tree", side: "left" })
    );

    expect(getByText("tree panel")).toBeInTheDocument();
    expect(container.get(QueryBus).query<string, IPanelSideCommand>(PANEL_ACTIVE_QUERY, { side: "left" })).toBe("tree");
  });

  it("offers the jobs listing beside the notification centre in dev mode", async () => {
    // Reachability rather than rendering: the panel is registered in one list and its button rendered from another, so
    // a panel can be perfectly correct and still have no way to open it. That is exactly how it shipped invisible.
    mockIpcCommands({ [EJobsCommand.LIST]: [] });

    const { getByLabelText, findByText } = renderFrame(<div>bare editor</div>);

    await userEvent.click(getByLabelText("Jobs"));

    expect(await findByText(/Nothing is running/)).toBeInTheDocument();
  });

  it("hides the jobs listing outside dev mode, where it answers nothing a person needs", () => {
    setLocalStorageValue("xrf.preference.dev-mode", "false");

    const { queryByLabelText } = renderFrame(<div>bare editor</div>);

    expect(queryByLabelText("Jobs")).toBeNull();
    expect(queryByLabelText("Notifications")).toBeInTheDocument();
  });

  it("offers the notification centre even when the active editor declares no panels", () => {
    const { getByLabelText } = renderFrame(<div>bare editor</div>);

    expect(getByLabelText("Notifications")).toBeInTheDocument();
  });
});
