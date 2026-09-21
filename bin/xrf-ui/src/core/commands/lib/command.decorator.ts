import { XrfApplicationError } from "@/core/error/lib";
import { Optional } from "@/lib/types/general";

import { IKeybindCommand } from "./command-descriptor";
import { appendKeybindCommandHandler } from "./command-metadata";

/** How a handler qualifies the keybind command it answers. */
export interface IKeybindCommandHandlerOptions<T> {
  /** Whether the command is legal right now, read from the service that owns the handler. */
  isEnabled?: (service: T) => boolean;
}

/**
 * Binds a service method to a declared keybind command for the lifetime of its container.
 *
 * @param descriptor - The declared command this method answers.
 * @param options - Enablement for this handler.
 * @returns The method decorator.
 */
export function KeybindCommand<T extends object>(
  descriptor: IKeybindCommand,
  options: IKeybindCommandHandlerOptions<T> = {}
): MethodDecorator {
  return (target: object, propertyKey: string | symbol): void => {
    if (typeof propertyKey !== "string") {
      throw new XrfApplicationError(`Keybind command '${descriptor.id}' must decorate a named method.`);
    }

    appendKeybindCommandHandler(target.constructor, {
      descriptor,
      isEnabled: options.isEnabled as Optional<(service: object) => boolean>,
      methodName: propertyKey,
    });
  };
}
