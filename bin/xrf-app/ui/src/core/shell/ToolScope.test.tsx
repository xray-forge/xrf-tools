import { describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { Injectable } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { Fragment, ReactElement, Suspense } from "react";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";
import { IEditorPanel, selectPanelsOnSide, useEditorPanels, useEditorPanelsRegistry } from "@/core/shell/editor-shell";
import { ToolScope } from "@/core/shell/ToolScope";
import { renderWithProviders } from "@/fixtures/utils/render";

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

const TOOL_METADATA = {
  description: "",
  group: EToolGroupId.ARCHIVES,
  icon: <span>a</span>,
  id: EToolId.ARCHIVES_EXPLORER,
  label: "Scoped tool",
  path: "/archives-explorer",
  status: EToolStatus.READY,
};

const TOOL = createToolDescriptor(TOOL_METADATA, {
  load: async () => ({ Component: Publisher, container: { bindings: [ScopedService] } }),
});

describe("ToolScope", () => {
  it("reaches the panels the shell renders, not just the tool's own tree", async () => {
    // The archives menu injects its service and is published as a panel. When the tool provided
    // its own container the panel rendered outside it and the injection threw, which is the whole
    // reason the shell owns the scope around both surfaces.
    const { findByText } = await act(async () =>
      renderWithProviders(
        <Suspense fallback={null}>
          <ToolScope tool={TOOL}>
            <Publisher />
            <PanelSlot />
          </ToolScope>
        </Suspense>,
        { hasShell: true }
      )
    );

    expect(await findByText("content")).toBeInTheDocument();
    expect(await findByText("scoped service")).toBeInTheDocument();
  });

  it("keeps the container when a rebuilt runtime binds the same classes", async () => {
    const TRACKED: IToolDescriptor = createToolDescriptor(TOOL_METADATA, {
      load: async () => ({ Component: TrackedPanel, container: { bindings: [TrackedService] } }),
    });

    const { findByText, rerender } = await act(async () =>
      renderWithProviders(
        <Suspense fallback={null}>
          <ToolScope tool={TRACKED}>
            <TrackedPanel />
          </ToolScope>
        </Suspense>
      )
    );

    const instance: Nullable<string> = (await findByText(/instance /)).textContent;
    const rebuilt: IToolDescriptor = createToolDescriptor(
      {
        ...TOOL_METADATA,
        label: "Updated tool label",
      },
      {
        load: async () => ({ Component: TrackedPanel, container: { bindings: [TrackedService] } }),
      }
    );

    await act(async () =>
      rerender(
        <>
          <Suspense fallback={null}>
            <ToolScope tool={rebuilt}>
              <TrackedPanel />
            </ToolScope>
          </Suspense>
        </>
      )
    );

    expect((await findByText(/instance /)).textContent).toBe(instance);
  });

  it("renders in the root container when no tool owns the route", () => {
    const { getByText } = renderWithProviders(
      <ToolScope tool={null}>
        <div>home</div>
      </ToolScope>
    );

    expect(getByText("home")).toBeInTheDocument();
  });
});
