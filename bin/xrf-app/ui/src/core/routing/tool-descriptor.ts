import { Optional } from "@xrf/types";
import { lazy } from "react";

import { IToolDescriptor, IToolMetadata, IToolRuntime } from "@/core/routing/tool";

/** Supply a ready runtime or a deferred loader; the factory owns preloading and lazy wrapping. */
export type TToolDescriptorOptions =
  (IToolRuntime & { load?: never }) | { load: () => Promise<IToolRuntime>; Component?: never; container?: never };

/**
 * Creates a synchronous or lazy tool descriptor from its metadata and runtime source.
 *
 * A supplied component is used directly. Preloading and rendering share one runtime load.
 * Preload failures are silent; loading the tool still exposes the original error.
 * Service containers are created by the shell when the tool mounts.
 *
 * @param metadata - Tool identity and presentation fields.
 * @param options - A component with optional bindings, or a deferred runtime loader.
 * @returns A descriptor ready for routing, service scoping, and preloading.
 */
export function createToolDescriptor(metadata: IToolMetadata, options: TToolDescriptorOptions): IToolDescriptor {
  if (!options.load) {
    return { ...metadata, ...options };
  }

  const loadRuntime = options.load;

  let pending: Optional<Promise<IToolRuntime>>;

  function load(): Promise<IToolRuntime> {
    return (pending ??= loadRuntime());
  }

  /**
   * Warms the runtime without reporting failures before the tool opens.
   *
   * @returns Completion of the preload attempt, whether it succeeds or fails.
   */
  async function preload(): Promise<void> {
    try {
      await load();
    } catch {
      // Keep the rejected runtime promise so opening the tool reaches its error boundary.
    }
  }

  return {
    ...metadata,
    Component: lazy(() => load().then((runtime) => ({ default: runtime.Component }))),
    load,
    preload,
  };
}
