import { describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { Injectable } from "@wirestate/core";
import { registerHotModule, requestHotSwap } from "@wirestate/core/hot";
import { useInjection } from "@wirestate/react";
import { Fragment, ReactElement } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { ApplicationScope } from "@/core/shell/ApplicationScope";
import {
  EditorPanelsProvider,
  IEditorPanel,
  selectPanelsOnSide,
  useEditorPanels,
  useEditorPanelsRegistry,
} from "@/core/shell/panel/context";
import { renderWithProviders } from "@/fixtures/utils/render";
import { noop } from "@/lib/callbacks/noop";
import { Nullable } from "@/lib/types/general";

@Injectable()
class ScopedService {
  public readonly label: string = "scoped service";
}

/** Stands in for `ScopedService` after a hot update re-executed its module. */
@Injectable()
class ReloadedScopedService {
  public readonly label: string = "reloaded service";
}

let nextInstanceId: number = 0;

@Injectable()
class TrackedService {
  public readonly id: number = ++nextInstanceId;
}

const PANEL: IEditorPanel = {
  icon: <span>p</span>,
  id: "scoped",
  label: "Scoped",
  render: () => <ScopedPanel />,
  side: "left",
};

function ScopedPanel(): ReactElement {
  const service: ScopedService = useInjection(ScopedService);

  return <div>{service.label}</div>;
}

function ReloadedScopedPanel(): ReactElement {
  const service: ReloadedScopedService = useInjection(ReloadedScopedService);

  return <div>{service.label}</div>;
}

function TrackedPanel(): ReactElement {
  const service: TrackedService = useInjection(TrackedService);

  return <div>{`instance ${service.id}`}</div>;
}

/** Publishes a panel and nothing else, the way an editor does. */
function Publisher(): ReactElement {
  useEditorPanels(() => [PANEL], []);

  return <div>content</div>;
}

/** Stands in for `ApplicationPanelSlot`, which is the thing that renders a published panel. */
function PanelSlot(): ReactElement {
  const panels: ReadonlyArray<IEditorPanel> = useEditorPanelsRegistry();

  return (
    <>
      {selectPanelsOnSide(panels, "left").map((panel: IEditorPanel) => (
        <Fragment key={panel.id}>{panel.render()}</Fragment>
      ))}
    </>
  );
}

const APPLICATION: IApplicationDescriptor = {
  container: { bindings: [ScopedService] },
  Component: Publisher,
  description: "",
  group: EApplicationGroupId.ARCHIVES,
  icon: <span>a</span>,
  id: EApplicationId.ARCHIVES_EXPLORER,
  label: "Scoped application",
  path: "/archives-explorer",
  status: EApplicationStatus.READY,
};

describe("ApplicationScope", () => {
  it("reaches the panels the shell renders, not just the application's own tree", () => {
    // The archives menu injects its service and is published as a panel. When the application provided
    // its own container the panel rendered outside it and the injection threw, which is the whole
    // reason bindings moved onto the descriptor.
    const { getByText } = renderWithProviders(
      <EditorPanelsProvider>
        <ApplicationScope application={APPLICATION}>
          <Publisher />
          <PanelSlot />
        </ApplicationScope>
      </EditorPanelsProvider>
    );

    expect(getByText("content")).toBeInTheDocument();
    expect(getByText("scoped service")).toBeInTheDocument();
  });

  it("rebuilds the container when hot reload replaces a bound class", async () => {
    registerHotModule("ApplicationScope.test/ScopedService", { ScopedService });

    const { getByText, rerender } = renderWithProviders(
      <ApplicationScope application={APPLICATION}>
        <ScopedPanel />
      </ApplicationScope>
    );

    expect(getByText("scoped service")).toBeInTheDocument();

    // A completed swap announces itself to the console exactly as it does during a dev session. That line is
    // wirestate reporting what it was asked to do, not anything failing here, so the run is not told about it.
    const reported: jest.SpiedFunction<Console["info"]> = jest.spyOn(console, "info").mockImplementation(noop);

    await act(async () => {
      registerHotModule("ApplicationScope.test/ScopedService", { ScopedService: ReloadedScopedService });
      requestHotSwap();
    });

    reported.mockRestore();

    rerender(
      <>
        <ApplicationScope application={APPLICATION}>
          <ReloadedScopedPanel />
        </ApplicationScope>
      </>
    );

    expect(getByText("reloaded service")).toBeInTheDocument();
  });

  it("keeps the container when a rebuilt descriptor binds the same classes", () => {
    // The common hot update, where a module above the descriptor re-executed but the service did not.
    // Rebuilding here would discard the live services and everything they hold open, so the same
    // instance has to come back out.
    const TRACKED: IApplicationDescriptor = { ...APPLICATION, container: { bindings: [TrackedService] } };

    const { getByText, rerender } = renderWithProviders(
      <ApplicationScope application={TRACKED}>
        <TrackedPanel />
      </ApplicationScope>
    );

    const instance: Nullable<string> = getByText(/instance /).textContent;

    rerender(
      <>
        <ApplicationScope application={{ ...TRACKED, container: { bindings: [TrackedService] } }}>
          <TrackedPanel />
        </ApplicationScope>
      </>
    );

    expect(getByText(/instance /).textContent).toBe(instance);
  });

  it("renders in the root container when no application owns the route", () => {
    const { getByText } = renderWithProviders(
      <ApplicationScope application={null}>
        <div>home</div>
      </ApplicationScope>
    );

    expect(getByText("home")).toBeInTheDocument();
  });
});
