import { WirestatePlugin } from "@wirestate/core";

import { cancelFlows } from "@/lib/mobx/flow/lanes";

/** Cancels service flows when their owning instance is released. */
export class FlowCancellationPlugin implements WirestatePlugin {
  /**
   * Abandons pending flows after the service's own deactivation hook.
   * Deprovision leaves them running because React can reprovision the same instance.
   *
   * @param instance - Service being released by its container.
   */
  public onDeactivate(instance: object): void {
    cancelFlows(instance);
  }
}
