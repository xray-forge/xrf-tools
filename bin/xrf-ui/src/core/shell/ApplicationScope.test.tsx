import { describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { Injectable } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { Fragment, ReactElement, Suspense } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";
import { ApplicationScope } from "@/core/shell/ApplicationScope";
import {
  EditorPanelsProvider,
  IEditorPanel,
  selectPanelsOnSide,
  useEditorPanels,
  useEditorPanelsRegistry,
} from "@/core/shell/panel/context";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

@Injectable()
class ScopedService {
  public readonly label: string = "scoped service";
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

const APPLICATION_METADATA = {
  description: "",
  group: EApplicationGroupId.ARCHIVES,
  icon: <span>a</span>,
  id: EApplicationId.ARCHIVES_EXPLORER,
  label: "Scoped application",
  path: "/archives-explorer",
  status: EApplicationStatus.READY,
};

const APPLICATION = createApplicationDescriptor(APPLICATION_METADATA, {
  load: async () => ({ Component: Publisher, container: { bindings: [ScopedService] } }),
});

describe("ApplicationScope", () => {
  it("reaches the panels the shell renders, not just the application's own tree", async () => {
    // The archives menu injects its service and is published as a panel. When the application provided
    // its own container the panel rendered outside it and the injection threw, which is the whole
    // reason the shell owns the scope around both surfaces.
    const { findByText } = await act(async () =>
      renderWithProviders(
        <EditorPanelsProvider>
          <Suspense fallback={null}>
            <ApplicationScope application={APPLICATION}>
              <Publisher />
              <PanelSlot />
            </ApplicationScope>
          </Suspense>
        </EditorPanelsProvider>
      )
    );

    expect(await findByText("content")).toBeInTheDocument();
    expect(await findByText("scoped service")).toBeInTheDocument();
  });

  it("keeps the container when a rebuilt runtime binds the same classes", async () => {
    const TRACKED: IApplicationDescriptor = createApplicationDescriptor(APPLICATION_METADATA, {
      load: async () => ({ Component: TrackedPanel, container: { bindings: [TrackedService] } }),
    });

    const { findByText, rerender } = await act(async () =>
      renderWithProviders(
        <Suspense fallback={null}>
          <ApplicationScope application={TRACKED}>
            <TrackedPanel />
          </ApplicationScope>
        </Suspense>
      )
    );

    const instance: Nullable<string> = (await findByText(/instance /)).textContent;
    const rebuilt: IApplicationDescriptor = createApplicationDescriptor(
      {
        ...APPLICATION_METADATA,
        label: "Updated application label",
      },
      {
        load: async () => ({ Component: TrackedPanel, container: { bindings: [TrackedService] } }),
      }
    );

    await act(async () =>
      rerender(
        <>
          <Suspense fallback={null}>
            <ApplicationScope application={rebuilt}>
              <TrackedPanel />
            </ApplicationScope>
          </Suspense>
        </>
      )
    );

    expect((await findByText(/instance /)).textContent).toBe(instance);
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
