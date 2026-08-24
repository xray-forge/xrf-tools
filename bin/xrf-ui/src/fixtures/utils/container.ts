import { Binding, Container, ServiceToken } from "@wirestate/core";

import { AssetService } from "@/core/assets/services";
import { createContainerPlugins } from "@/core/container";
import { ProjectService } from "@/core/settings/services/project";

export interface IInjectedServiceMockDescriptor<T> {
  service: T;
  container: Container;
}

/**
 * Builds a container wired the way the application wires one.
 *
 * @param bindings - Services to register.
 * @returns A container ready to resolve them.
 */
export function mockContainer(bindings: Array<Binding>): Container {
  return new Container({ bindings, plugins: createContainerPlugins() });
}

/**
 * Builds a service through the same container path as the application.
 *
 * Services that resolve dependencies with `inject()` cannot be constructed with `new`: there is no
 * injection context, and the call throws. Resolving without provisioning on purpose, so `@OnProvision`
 * does not fire and a test still sees a service that has asked the backend for nothing.
 *
 * @param token - Service token to resolve.
 * @param bindings - Additional bindings to register before the service token.
 * @returns The resolved service and its container.
 */
export function mockInjectedService<T>(
  token: ServiceToken<T>,
  bindings: Array<Binding> = []
): IInjectedServiceMockDescriptor<T> {
  const container: Container = mockContainer([AssetService, ProjectService, ...bindings, token as Binding]);

  return {
    container,
    service: container.get(token),
  };
}
