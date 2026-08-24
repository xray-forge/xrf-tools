import { WirestatePlugin, WireStatus } from "@wirestate/core";
import { makeObservable } from "@wirestate/mobx";

import { hasObservableMembers } from "@/lib/mobx/annotations";
import { cancelFlows } from "@/lib/mobx/flow";

/**
 * Applies MobX annotations to every activated service, so a constructor does not have to.
 *
 * Registered once per container. `makeObservable` runs at activation, which is after construction and after field
 * initializers, and before the instance's own `@OnActivation`. Services resolved without being provisioned - which is
 * how the test fixtures build them - are covered too, because activation happens on resolution.
 *
 * Every instance is lifecycle tracked here as well, so async work anywhere in a service can ask whether its owner is
 * still alive without the service having to hold a `WireStatus` of its own.
 */
export class ObservablePlugin implements WirestatePlugin {
  /**
   * Makes an activated instance observable when it has anything to make observable.
   *
   * @param instance - The activated instance.
   */
  public onActivate(instance: object): void {
    WireStatus.track(instance);

    if (hasObservableMembers(instance)) {
      makeObservable(instance);
    }
  }

  /**
   * Abandons whatever flows a service left running, after its own `@OnDeactivation`.
   *
   * Deactivation rather than deprovision, deliberately. React reaches deprovision and provision on every strict mode
   * remount while the instance itself survives, so cancelling there would abandon work the user is still waiting on.
   * Deactivation is the point the container actually lets the instance go.
   *
   * @param instance - The instance being deactivated.
   */
  public onDeactivate(instance: object): void {
    cancelFlows(instance);
  }
}
