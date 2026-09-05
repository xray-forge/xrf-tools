import { lazy } from "react";

import { IApplicationDescriptor, IApplicationMetadata, IApplicationRuntime } from "@/core/routing/application";
import { Optional } from "@/lib/types/general";

/** Supply a ready runtime or a deferred loader; the factory owns preloading and lazy wrapping. */
export type TApplicationDescriptorOptions =
  | (IApplicationRuntime & { load?: never })
  | { load: () => Promise<IApplicationRuntime>; Component?: never; container?: never };

/**
 * Creates a synchronous or lazy application descriptor from its metadata and runtime source.
 *
 * A supplied component is used directly. A loader is called only on preload or render, and both share its promise.
 * Service containers are created by the shell when the application mounts.
 *
 * @param metadata - Application identity and presentation fields.
 * @param options - A component with optional bindings, or a deferred runtime loader.
 * @returns A descriptor ready for routing, service scoping, and preloading.
 */
export function createApplicationDescriptor(
  metadata: IApplicationMetadata,
  options: TApplicationDescriptorOptions
): IApplicationDescriptor {
  if (!options.load) {
    return { ...metadata, ...options };
  }

  const loadRuntime = options.load;

  let pending: Optional<Promise<IApplicationRuntime>>;

  function load(): Promise<IApplicationRuntime> {
    return (pending ??= loadRuntime());
  }

  return {
    ...metadata,
    Component: lazy(() => load().then((runtime) => ({ default: runtime.Component }))),
    load,
    preload: load,
  };
}
