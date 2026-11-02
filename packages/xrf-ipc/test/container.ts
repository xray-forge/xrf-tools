import { Binding, Container, EventsPlugin, ServiceToken } from "@wirestate/core";
import { ObservablePlugin } from "@wirestate/mobx";

/**
 * A container with the plugins the session layer's services rely on: events for the changes they announce, and
 * observables for the state they mirror.
 *
 * @param bindings - Services to register.
 * @returns A container ready to resolve them.
 */
export function mockContainer(bindings: Array<Binding> = []): Container {
  return new Container({ bindings, plugins: [new EventsPlugin(), new ObservablePlugin()] });
}

/**
 * Resolves a service through a container, without provisioning it, so `@OnProvision` asks the backend for nothing.
 *
 * @param token - Service token to resolve.
 * @param bindings - Further bindings it needs.
 * @returns The resolved service and its container.
 */
export function mockInjectedService<T>(
  token: ServiceToken<T>,
  bindings: Array<Binding> = []
): { service: T; container: Container } {
  const container: Container = mockContainer([...bindings, token as Binding]);

  return { container, service: container.get(token) };
}
