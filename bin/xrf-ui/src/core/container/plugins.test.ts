import { describe, expect, it } from "@jest/globals";
import { Binding, Container, Newable, ServiceToken } from "@wirestate/core";
import { isObservableObject } from "@wirestate/mobx";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { createContainerPlugins } from "@/core/container";
import { ErrorCaptureService } from "@/core/notifications/services/error-capture.service";
import { NotificationsService } from "@/core/notifications/services/notifications.service";
import { ProjectService } from "@/core/settings/services/project";
import { SettingsService } from "@/core/settings/services/settings";
import { hasObservableMembers } from "@/lib/mobx";

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
  const bound: Set<Binding> = new Set<Binding>([
    ProjectService,
    SettingsService,
    NotificationsService,
    ErrorCaptureService,
  ]);

  for (const application of APPLICATION_CATALOG.applications) {
    for (const binding of application.container?.bindings ?? []) {
      bound.add(binding);
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
