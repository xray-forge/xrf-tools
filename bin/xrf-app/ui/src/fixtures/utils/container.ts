import { Binding, Container, ServiceToken } from "@wirestate/core";
import { SESSION_TOOL, SessionScopeService } from "@xrf/ipc/session";

import { AssetService } from "@/core/assets/services";
import { createContainerPlugins, ROOT_BINDINGS } from "@/core/container";

export interface IInjectedServiceMockDescriptor<T> {
  service: T;
  container: Container;
}

/**
 * Builds a container wired the way the tool wires one.
 *
 * @param bindings - Services to register beside the root ones; one naming a root binding's token replaces it.
 * @returns A container ready to resolve them.
 */
export function mockContainer(bindings: Array<Binding> = []): Container {
  const tokens: Set<unknown> = new Set(bindings.map(toBindingToken));
  const roots: Array<Binding> = ROOT_BINDINGS.filter((it: Binding) => !tokens.has(toBindingToken(it)));

  return new Container({ bindings: [...roots, ...bindings], plugins: createContainerPlugins() });
}

/**
 * The bindings a tool's container adds over the root, as `ApplicationScope` adds them: the tool's id as
 * the tool its openings are held for and its notices name, and a scope of its own.
 *
 * @param tool - The tool's id.
 * @returns The bindings to pass beside the service under test.
 */
export function mockToolScope(tool: string): Array<Binding> {
  return [{ token: SESSION_TOOL, value: tool }, SessionScopeService];
}

/**
 * Builds a service through the same container path as the tool.
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
  const container: Container = mockContainer([AssetService, ...bindings, token as Binding]);

  return {
    container,
    service: container.get(token),
  };
}

function toBindingToken(binding: Binding): unknown {
  return typeof binding === "function" ? binding : binding.token;
}
