import { CommandBus, Container, QueryBus, ServiceToken, WirestatePlugin } from "@wirestate/core";

import { toKeybindCommandEnabledQuery } from "@/core/commands/lib/command-descriptor";
import {
  collectKeybindCommandHandlers,
  hasKeybindCommandHandlers,
  IKeybindCommandHandlerMetadata,
} from "@/core/commands/lib/command-metadata";
import { XrfApplicationError } from "@/core/error/lib";

/**
 * Wires `@KeybindCommand` methods onto the command and query buses for one provision cycle.
 */
export class KeybindCommandBindingPlugin implements WirestatePlugin {
  /**
   * Force-activates any service declaring commands, so a service that exists only to answer them is still wired.
   *
   * @param token - Binding token to inspect.
   * @returns Whether the token declares keybind command handlers.
   */
  public participates(token: ServiceToken): boolean {
    return hasKeybindCommandHandlers(token);
  }

  /**
   * Registers the instance's handlers and guards, and unregisters both at deprovision.
   *
   * @param instance - Service being provisioned.
   * @param container - Container provisioning it.
   * @param addDisposer - Registers teardown for this provision cycle.
   */
  public onProvision(instance: object, container: Container, addDisposer: (dispose: () => void) => void): void {
    const handlers: ReadonlyArray<IKeybindCommandHandlerMetadata> = collectKeybindCommandHandlers(instance);

    if (handlers.length === 0) {
      return;
    }

    const commandBus: CommandBus = container.get(CommandBus);
    const queryBus: QueryBus = container.get(QueryBus);

    for (const handler of handlers) {
      const method: unknown = (instance as Record<string, unknown>)[handler.methodName];

      if (typeof method !== "function") {
        throw new XrfApplicationError(
          `Keybind command '${handler.descriptor.id}' names '${handler.methodName}' on ` +
            `'${instance.constructor.name}', which is not a method.`
        );
      }

      addDisposer(commandBus.register(handler.descriptor.id, (method as (payload: unknown) => unknown).bind(instance)));
      addDisposer(
        queryBus.register(toKeybindCommandEnabledQuery(handler.descriptor), () => handler.isEnabled?.(instance) ?? true)
      );
    }
  }
}
