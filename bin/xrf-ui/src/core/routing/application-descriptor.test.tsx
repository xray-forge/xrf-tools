import { describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { Injectable } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { ReactElement, Suspense } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
  IApplicationRuntime,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";
import { CurrentApplicationProvider } from "@/core/routing/current-application.context";
import { ApplicationScope } from "@/core/shell/ApplicationScope";
import { ApplicationShell } from "@/core/shell/ApplicationShell";
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

const RUNTIME: IApplicationRuntime = { Component: RuntimeContent, container: { bindings: [RuntimeService] } };
const METADATA = {
  description: "Lazy runtime fixture",
  group: EApplicationGroupId.ARCHIVES,
  icon: <span />,
  id: EApplicationId.ARCHIVES_EXPLORER,
  label: "Runtime fixture",
  path: "/archives-explorer",
  status: EApplicationStatus.READY,
};

describe("createApplicationDescriptor", () => {
  it("uses a synchronous component and bindings without a loading boundary", () => {
    const application = createApplicationDescriptor(METADATA, RUNTIME);

    expect(application.Component).toBe(RuntimeContent);
    expect(application.container).toBe(RUNTIME.container);
    expect(application.load).toBeUndefined();
    expect(application.preload).toBeUndefined();
    expect(constructed).not.toHaveBeenCalled();

    const view = renderWithProviders(
      <ApplicationScope application={application}>
        <application.Component />
      </ApplicationScope>
    );

    expect(view.getByText("Loaded runtime service")).toBeInTheDocument();
    expect(constructed).toHaveBeenCalledTimes(1);
  });

  it("supports synchronous components that only use the root container", () => {
    function RootContent(): ReactElement {
      return <div>Root application</div>;
    }

    const application = createApplicationDescriptor(METADATA, {
      Component: RootContent,
    });
    const view = renderWithProviders(
      <ApplicationScope application={application}>
        <application.Component />
      </ApplicationScope>
    );

    expect(view.getByText("Root application")).toBeInTheDocument();
    expect(application.container).toBeUndefined();
    expect(application.load).toBeUndefined();
  });

  it("shares preloading and rendering without instantiating services during preload", async () => {
    const loadRuntime = jest.fn(async () => RUNTIME);
    const application = createApplicationDescriptor(METADATA, {
      load: loadRuntime,
    });

    expect(loadRuntime).not.toHaveBeenCalled();

    const preloaded = application.preload?.();
    const loaded = application.load?.();

    expect(application.load?.()).toBe(loaded);
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
    const application = createApplicationDescriptor(METADATA, { load: loadRuntime });

    await expect(application.preload?.()).resolves.toBeUndefined();
    await expect(application.preload?.()).resolves.toBeUndefined();

    const loaded = application.load?.();

    expect(application.load?.()).toBe(loaded);
    await expect(loaded).rejects.toBe(error);
    expect(loadRuntime).toHaveBeenCalledTimes(1);
    expect(constructed).not.toHaveBeenCalled();
  });

  it("also contains a loader that throws before returning a promise during preload", async () => {
    const error = new Error("Loader unavailable");
    const application = createApplicationDescriptor(METADATA, {
      load: () => {
        throw error;
      },
    });

    await expect(application.preload?.()).resolves.toBeUndefined();
    expect(() => application.load?.()).toThrow(error);
  });

  it("waits for bindings before rendering the component and uses the same load", async () => {
    let resolveRuntime: (runtime: IApplicationRuntime) => void = noop;
    const pending = new Promise<IApplicationRuntime>((resolve) => {
      resolveRuntime = resolve;
    });
    const loadRuntime = jest.fn(() => pending);
    const application = createApplicationDescriptor(METADATA, {
      load: loadRuntime,
    });
    const view = await act(async () =>
      renderWithProviders(
        <Suspense fallback={<div>Loading runtime</div>}>
          <ApplicationScope application={application}>
            <application.Component />
          </ApplicationScope>
        </Suspense>
      )
    );

    expect(view.getByText("Loading runtime")).toBeInTheDocument();
    expect(constructed).not.toHaveBeenCalled();

    await act(async () => resolveRuntime(RUNTIME));

    expect(view.getByText("Loaded runtime service")).toBeInTheDocument();
    expect(view.queryByText("Loading runtime")).not.toBeInTheDocument();
    expect(loadRuntime).toHaveBeenCalledTimes(1);
    expect(constructed).toHaveBeenCalledTimes(1);
  });

  it.each([false, true])(
    "keeps window controls available after a runtime failure (preloaded: %s)",
    async (isPreloaded) => {
      const application: IApplicationDescriptor = createApplicationDescriptor(METADATA, {
        load: async () => {
          throw new Error("Runtime chunk unavailable");
        },
      });

      if (isPreloaded) {
        await application.preload?.();
      }

      const consoleError = jest.spyOn(console, "error").mockImplementation(noop);

      try {
        const view = await act(async () =>
          renderWithProviders(
            <CurrentApplicationProvider application={application}>
              <ApplicationShell>
                <application.Component />
              </ApplicationShell>
            </CurrentApplicationProvider>
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
