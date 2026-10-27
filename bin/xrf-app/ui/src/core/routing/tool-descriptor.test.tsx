import { describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { Injectable } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { ReactElement, Suspense } from "react";

import { CurrentToolProvider } from "@/core/routing/current-tool.context";
import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor, IToolRuntime } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";
import { ApplicationShell } from "@/core/shell/ApplicationShell";
import { ToolScope } from "@/core/shell/ToolScope";
import { deferred } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import { noop } from "@/lib/callbacks/noop";

const constructed = jest.fn();

@Injectable()
class RuntimeService {
  public readonly label: string = "Loaded runtime service";

  public constructor() {
    constructed();
  }
}

function RuntimeContent(): ReactElement {
  const service: RuntimeService = useInjection(RuntimeService);

  return <div>{service.label}</div>;
}

const RUNTIME: IToolRuntime = { Component: RuntimeContent, container: { bindings: [RuntimeService] } };
const METADATA = {
  description: "Lazy runtime fixture",
  group: EToolGroupId.ARCHIVES,
  icon: <span />,
  id: EToolId.ARCHIVES_EXPLORER,
  label: "Runtime fixture",
  path: "/archives-explorer",
  status: EToolStatus.READY,
};

describe("createToolDescriptor", () => {
  it("uses a synchronous component and bindings without a loading boundary", () => {
    const tool = createToolDescriptor(METADATA, RUNTIME);

    expect(tool.Component).toBe(RuntimeContent);
    expect(tool.container).toBe(RUNTIME.container);
    expect(tool.load).toBeUndefined();
    expect(tool.preload).toBeUndefined();
    expect(constructed).not.toHaveBeenCalled();

    const view = renderWithProviders(
      <ToolScope tool={tool}>
        <tool.Component />
      </ToolScope>
    );

    expect(view.getByText("Loaded runtime service")).toBeInTheDocument();
    expect(constructed).toHaveBeenCalledTimes(1);
  });

  it("supports synchronous components that only use the root container", () => {
    function RootContent(): ReactElement {
      return <div>Root tool</div>;
    }

    const tool = createToolDescriptor(METADATA, {
      Component: RootContent,
    });
    const view = renderWithProviders(
      <ToolScope tool={tool}>
        <tool.Component />
      </ToolScope>
    );

    expect(view.getByText("Root tool")).toBeInTheDocument();
    expect(tool.container).toBeUndefined();
    expect(tool.load).toBeUndefined();
  });

  it("shares preloading and rendering without instantiating services during preload", async () => {
    const loadRuntime = jest.fn(async () => RUNTIME);
    const tool = createToolDescriptor(METADATA, {
      load: loadRuntime,
    });

    expect(loadRuntime).not.toHaveBeenCalled();

    const preloaded = tool.preload?.();
    const loaded = tool.load?.();

    expect(tool.load?.()).toBe(loaded);
    await expect(preloaded).resolves.toBeUndefined();
    await expect(loaded).resolves.toBe(RUNTIME);
    expect(loadRuntime).toHaveBeenCalledTimes(1);
    expect(constructed).not.toHaveBeenCalled();
  });

  it("settles failed preloads silently while preserving the runtime error", async () => {
    const error = new Error("Runtime chunk unavailable");
    const loadRuntime = jest.fn(async () => {
      throw error;
    });
    const tool = createToolDescriptor(METADATA, { load: loadRuntime });

    await expect(tool.preload?.()).resolves.toBeUndefined();
    await expect(tool.preload?.()).resolves.toBeUndefined();

    const loaded = tool.load?.();

    expect(tool.load?.()).toBe(loaded);
    await expect(loaded).rejects.toBe(error);
    expect(loadRuntime).toHaveBeenCalledTimes(1);
    expect(constructed).not.toHaveBeenCalled();
  });

  it("also contains a loader that throws before returning a promise during preload", async () => {
    const error = new Error("Loader unavailable");
    const tool = createToolDescriptor(METADATA, {
      load: () => {
        throw error;
      },
    });

    await expect(tool.preload?.()).resolves.toBeUndefined();
    expect(() => tool.load?.()).toThrow(error);
  });

  it("waits for bindings before rendering the component and uses the same load", async () => {
    const pending = deferred<IToolRuntime>();
    const loadRuntime = jest.fn(() => pending.promise);
    const tool = createToolDescriptor(METADATA, {
      load: loadRuntime,
    });
    const view = await act(async () =>
      renderWithProviders(
        <Suspense fallback={<div>Loading runtime</div>}>
          <ToolScope tool={tool}>
            <tool.Component />
          </ToolScope>
        </Suspense>
      )
    );

    expect(view.getByText("Loading runtime")).toBeInTheDocument();
    expect(constructed).not.toHaveBeenCalled();

    await act(async () => pending.resolve(RUNTIME));

    expect(view.getByText("Loaded runtime service")).toBeInTheDocument();
    expect(view.queryByText("Loading runtime")).not.toBeInTheDocument();
    expect(loadRuntime).toHaveBeenCalledTimes(1);
    expect(constructed).toHaveBeenCalledTimes(1);
  });

  it.each([false, true])(
    "keeps window controls available after a runtime failure (preloaded: %s)",
    async (isPreloaded) => {
      const tool: IToolDescriptor = createToolDescriptor(METADATA, {
        load: async () => {
          throw new Error("Runtime chunk unavailable");
        },
      });

      if (isPreloaded) {
        await tool.preload?.();
      }

      const consoleError = jest.spyOn(console, "error").mockImplementation(noop);

      try {
        const view = await act(async () =>
          renderWithProviders(
            <CurrentToolProvider tool={tool}>
              <ApplicationShell>
                <tool.Component />
              </ApplicationShell>
            </CurrentToolProvider>,
            { hasShell: true }
          )
        );

        expect(view.getByText("This tool stopped rendering")).toBeInTheDocument();
        expect(view.getByTestId("application-title-bar")).toBeInTheDocument();
        expect(view.getByRole("button", { name: "Notifications" })).toBeInTheDocument();
        expect(view.getByRole("button", { name: "Go home" })).toBeInTheDocument();
      } finally {
        consoleError.mockRestore();
      }
    }
  );
});
