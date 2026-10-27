import { describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { Injectable } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { Logger } from "@xrf/logging";

import { SessionTool } from "@/core/session/components/SessionTool";
import { mockHeldSessionRoot, mockLetGoSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService, mockToolScope } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

@Injectable()
class HolderService {
  public readonly log: Logger = new Logger("holder");

  public readonly root: SessionRootResource<string> = new SessionRootResource(this.log, {
    subject: "test project",
    close: jest.fn<(sessionIds: Array<SessionId>) => Promise<unknown>>(),
  });
}

function setup() {
  const { container, service } = mockInjectedService(HolderService, mockToolScope("exports-explorer"));

  runInAction(() => {
    service.root.isRestored = false;
  });

  return { container, root: service.root };
}

describe("SessionTool", () => {
  it("shows the loader until the backend has said what the tool holds, then the open form", () => {
    const { container, root } = setup();
    const { getByText, queryByText } = renderWithProviders(
      <SessionTool root={root} openForm={<div>open form</div>} workspace={<div>workspace</div>} />,
      { container }
    );

    // A reloaded page waits rather than flashing the open form before what was open comes back.
    expect(queryByText("open form")).toBeNull();
    expect(queryByText("workspace")).toBeNull();

    act(() =>
      runInAction(() => {
        root.isRestored = true;
      })
    );

    expect(getByText("open form")).toBeInTheDocument();
  });

  it("shows the workspace while the root holds a value, and the open form once it is let go", () => {
    const { container, root } = setup();

    mockHeldSessionRoot(root, mockSessionSnapshot("project"));

    const { getByText } = renderWithProviders(
      <SessionTool root={root} openForm={<div>open form</div>} workspace={<div>workspace</div>} />,
      { container }
    );

    expect(getByText("workspace")).toBeInTheDocument();

    act(() => mockLetGoSessionRoot(root));

    expect(getByText("open form")).toBeInTheDocument();
  });

  it("takes the tool's own word for what is open over the root's value", () => {
    const { container, root } = setup();

    mockHeldSessionRoot(root, mockSessionSnapshot("project"));

    const { getByText } = renderWithProviders(
      <SessionTool root={root} isOpen={false} openForm={<div>open form</div>} workspace={<div>workspace</div>} />,
      { container }
    );

    expect(getByText("open form")).toBeInTheDocument();
  });
});
