import { describe, expect, it } from "@jest/globals";
import { Container, Injectable, OnDeactivation, OnProvision, WireStatus } from "@wirestate/core";
import { flowResult, isObservableProp, Observable, reaction } from "@wirestate/mobx";

import { noop } from "@/lib/callbacks/noop";
import { call, LatestFlow, TFlow } from "@/lib/mobx";

import { createContainerPlugins } from "./plugins";

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
