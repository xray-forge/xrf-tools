import { describe, expect, it } from "@jest/globals";
import { Binding, BindingType, Container, getBindingType, Newable, ServiceToken } from "@wirestate/core";
import { isObservableObject } from "@wirestate/mobx";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { hasObservableMembers } from "@/lib/mobx";

import { ROOT_BINDINGS } from "./bindings";
import { createContainerPlugins } from "./plugins";

/**
 * Every service the shipped applications bind, deduplicated.
 *
 * Read off the catalog rather than listed here, so a service added to an application is covered without anyone
 * remembering to add it below.
 *
 * @returns Each bound service token, once.
 */
function catalogServices(): Array<Binding> {
  // The root container's own bindings come first: an application service may inject one, and resolving it here has to
  // go through the same graph the application would.
  const bound: Set<Binding> = new Set<Binding>(ROOT_BINDINGS);

  for (const application of APPLICATION_CATALOG.applications) {
    for (const binding of application.container?.bindings ?? []) {
      // Instance bindings only, in wirestate's own vocabulary. A factory binding names a token rather than a service —
      // an application pointing the shared inspection panels at whichever of its own services answers them — and what
      // it resolves to is a class already in this list.
      if (getBindingType(binding) === BindingType.Instance) {
        bound.add(binding);
      }
    }
  }

  return [...bound];
}

describe("createContainerPlugins", () => {
  it("makes every service the catalog binds observable", () => {
    const services: Array<Binding> = catalogServices();

    // Guards the failure this plugin can cause and lint cannot see: a container that omits it resolves services whose
    // annotations were never applied, so their state is inert and every screen over it silently stops updating.
    expect(services.length).toBeGreaterThan(0);

    for (const service of services) {
      const container: Container = new Container({
        bindings: [...services],
        plugins: createContainerPlugins(),
      });

      const instance: object = container.get(service as ServiceToken<object>);
      const name: string = (service as Newable<object>).name;

      // A service with nothing annotated is left alone on purpose, so the invariant is the pair rather than the flag:
      // whatever declares observable members must come out observable.
      expect({ name, observable: isObservableObject(instance) }).toEqual({
        name,
        observable: hasObservableMembers(instance),
      });
    }
  });
});
