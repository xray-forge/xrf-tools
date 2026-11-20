import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { Injectable } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { PropsWithChildren, ReactElement } from "react";
import { NavigateFunction, Route, Routes, useLocation, useNavigate } from "react-router-dom";

import { NotificationsService } from "@/core/notifications/services";
import { CurrentToolProvider } from "@/core/routing/current-tool.context";
import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";
import { useShellPanels, useShellStatus } from "@/core/shell/contributions";
import { ApplicationShellFrame } from "@/core/shell/frame/ApplicationShellFrame";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

@Injectable()
class OutgoingToolService {
  public readonly label: string = "outgoing panel";
}

function OutgoingScopedPanel(): ReactElement {
  const service: OutgoingToolService = useInjection(OutgoingToolService);

  return <div>{service.label}</div>;
}

function OutgoingEditor(): ReactElement {
  const navigate: NavigateFunction = useNavigate();

  useShellStatus(["Outgoing status"]);

  useShellPanels(
    () => [
      {
        icon: <span>o</span>,
        id: "outgoing-menu",
        isOpenByDefault: true,
        label: "Outgoing",
        render: () => <OutgoingScopedPanel />,
        side: "left",
      },
    ],
    []
  );

  return <button onClick={() => navigate("/spawn-editor", { replace: true })}>leave</button>;
}

const TOOL_METADATA = { description: "", icon: <span>t</span>, label: "Tool", status: EToolStatus.READY };

const OUTGOING_TOOL: IToolDescriptor = createToolDescriptor(
  { ...TOOL_METADATA, group: EToolGroupId.ARCHIVES, id: EToolId.ARCHIVES_EXPLORER, path: "/archives-explorer" },
  { load: async () => ({ Component: OutgoingEditor, container: { bindings: [OutgoingToolService] } }) }
);

const INCOMING_TOOL: IToolDescriptor = createToolDescriptor(
  { ...TOOL_METADATA, group: EToolGroupId.SPAWNS, id: EToolId.SPAWN_EDITOR, path: "/spawn-editor" },
  { load: async () => ({ Component: () => <div>incoming editor</div>, container: { bindings: [] } }) }
);

/** Resolves the open tool from the route among the two stand-ins, as the catalog does among the real tools. */
function StandInTools({ children }: PropsWithChildren): ReactElement {
  const { pathname } = useLocation();
  const tool: Nullable<IToolDescriptor> =
    [OUTGOING_TOOL, INCOMING_TOOL].find((it: IToolDescriptor) => pathname.startsWith(it.path)) ?? null;

  return <CurrentToolProvider tool={tool}>{children}</CurrentToolProvider>;
}

describe("panel handover between tools", () => {
  beforeEach(() => clearLocalStorage());

  it("stops rendering a tool's panels the moment its container goes away", async () => {
    // The registry is cleared by an effect, but the container is swapped during render. For the commit
    // in between, the frame held the outgoing tool's panels and the incoming one's container -
    // so a panel that injects asked a container that never bound its service.
    const { getByTestId, getByText, findByText, queryByText } = await act(async () =>
      renderWithProviders(
        <StandInTools>
          <ApplicationShellFrame>
            <Routes>
              <Route path={"/archives-explorer/*"} element={<OutgoingEditor />} />
              <Route path={"/spawn-editor/*"} element={<div>incoming editor</div>} />
            </Routes>
          </ApplicationShellFrame>
        </StandInTools>,
        { bindings: [NotificationsService], route: "/archives-explorer", hasShell: true }
      )
    );

    expect(await findByText("outgoing panel")).toBeInTheDocument();
    expect(getByText("Outgoing status")).toBeInTheDocument();

    await act(async () => fireEvent.click(getByText("leave")));

    expect(await findByText("incoming editor")).toBeInTheDocument();
    expect(queryByText("outgoing panel")).not.toBeInTheDocument();
    expect(queryByText("Outgoing status")).not.toBeInTheDocument();
    expect(getByTestId("application-status-bar")).toBeEmptyDOMElement();
  });
});
