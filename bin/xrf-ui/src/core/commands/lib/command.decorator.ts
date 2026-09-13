import { ICommandDescriptor } from "@/core/commands/lib/command-descriptor";
import { appendCommandHandler } from "@/core/commands/lib/command-metadata";
import { Optional } from "@/lib/types/general";

/** How a handler qualifies the command it answers. */
export interface ICommandHandlerOptions<T> {
  /** Whether the command is legal right now, read from the service that owns the handler. */
  isEnabled?: (service: T) => boolean;
}

/**
 * Binds a service method to a declared command for the lifetime of its container.
 *
 * @param descriptor - The declared command this method answers.
 * @param options - Enablement for this handler.
 * @returns The method decorator.
 */
export function Command<T extends object>(
  descriptor: ICommandDescriptor,
  options: ICommandHandlerOptions<T> = {}
): MethodDecorator {
  return (target: object, propertyKey: string | symbol): void => {
    if (typeof propertyKey !== "string") {
      throw new Error(`Command '${descriptor.id}' must decorate a named method.`);
    }

    appendCommandHandler(target.constructor, {
      descriptor,
      isEnabled: options.isEnabled as Optional<(service: object) => boolean>,
      methodName: propertyKey,
    });
  };
}
