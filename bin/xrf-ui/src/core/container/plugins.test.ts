import { describe, expect, it } from "@jest/globals";
import {
  Binding,
  BindingType,
  Container,
  getBindingType,
  Injectable,
  OnDeactivation,
  OnProvision,
  ServiceToken,
  WireStatus,
} from "@wirestate/core";
import { flowResult, isObservableProp, Observable, reaction } from "@wirestate/mobx";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { noop } from "@/lib/callbacks/noop";
import { call, LatestFlow, TFlow } from "@/lib/mobx";

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
async function catalogServices(): Promise<Array<Binding>> {
  // The root container's own bindings come first: an application service may inject one, and resolving it here has to
  // go through the same graph the application would.
  const bound: Set<Binding> = new Set<Binding>(ROOT_BINDINGS);

  for (const application of APPLICATION_CATALOG.applications) {
    const runtime = application.load ? await application.load() : application;

    for (const binding of runtime?.container?.bindings ?? []) {
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

@Injectable()
class PluginTestService {
  @Observable()
  public result: number = 0;

  public readonly events: Array<string> = [];

  @OnProvision()
  public async onProvision(): Promise<void> {
    this.events.push("provision");
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.events.push("deactivation");
  }

  @LatestFlow()
  public *load(response: Promise<number>): TFlow {
    try {
      this.result = yield* call(response);
    } finally {
      this.events.push("settled");
    }
  }
}

describe("createContainerPlugins", () => {
  it("activates and lifecycle-tracks every service the catalog binds", async () => {
    const services: Array<Binding> = await catalogServices();

    expect(services.length).toBeGreaterThan(0);

    for (const service of services) {
      const container: Container = new Container({
        bindings: [...services],
        plugins: createContainerPlugins(),
      });

      const instance: object = container.get(service as ServiceToken<object>);

      expect(WireStatus.for(instance).isDeactivated).toBe(false);
    }
  });

  it("makes service changes observable through inherited container plugins", async () => {
    const parent = new Container({ plugins: createContainerPlugins() });
    const child = new Container({ parent, bindings: [PluginTestService] });
    const service = child.get(PluginTestService);
    const values: Array<number> = [];
    const dispose = reaction(
      () => service.result,
      (value) => values.push(value)
    );

    expect(isObservableProp(service, "result")).toBe(true);

    await service.load(Promise.resolve(7));

    expect(values).toEqual([7]);

    dispose();
    parent.destroy();
  });

  it("preserves pending flows across reprovision and cancels them after deactivation", async () => {
    const container = new Container({ bindings: [PluginTestService], plugins: createContainerPlugins() });
    const service = container.get(PluginTestService);
    const status = WireStatus.for(service);

    let resolve: (value: number) => void = noop;

    const response = new Promise<number>((settle) => {
      resolve = settle;
    });

    container.provision();

    const firstCycle = status.provisionId;
    const loading = flowResult(service.load(response));

    container.deprovision();
    container.provision();

    expect(status.isStale(firstCycle)).toBe(true);
    expect(service.events).toEqual(["provision", "provision"]);

    container.destroy();
    await loading;

    expect(service.events).toEqual(["provision", "provision", "deactivation", "settled"]);
    expect(status.isDeactivated).toBe(true);

    resolve(42);
    await response;

    expect(service.result).toBe(0);
  });
});
