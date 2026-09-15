import { IKeybindCommand } from "@/core/commands/lib/command-descriptor";
import { Nullable, Optional } from "@/lib/types/general";

/** One decorated method, as recorded against the class that declares it. */
export interface IKeybindCommandHandlerMetadata {
  readonly descriptor: IKeybindCommand;
  readonly methodName: string;
  /** Reads the guard from the instance that owns the handler; absent means always enabled. */
  readonly isEnabled?: (service: object) => boolean;
}

/**
 * Keyed by constructor rather than prototype, so a subclass adding handlers does not mutate its parent's list. The
 * chain walk below is what makes an inherited handler still register.
 */
const COMMAND_HANDLERS: WeakMap<object, Array<IKeybindCommandHandlerMetadata>> = new WeakMap();

/**
 * Records one decorated method against its declaring class.
 *
 * @param constructor - Class the decorated method belongs to.
 * @param metadata - Handler to record.
 */
export function appendKeybindCommandHandler(constructor: object, metadata: IKeybindCommandHandlerMetadata): void {
  const existing: Optional<Array<IKeybindCommandHandlerMetadata>> = COMMAND_HANDLERS.get(constructor);

  if (existing) {
    existing.push(metadata);
  } else {
    COMMAND_HANDLERS.set(constructor, [metadata]);
  }
}

/**
 * Collects the handlers a class and its ancestors declare, nearest class first.
 *
 * @param constructor - Class to inspect.
 * @returns Every handler reachable from that class.
 */
function collectFromConstructor(constructor: object): Array<IKeybindCommandHandlerMetadata> {
  const collected: Array<IKeybindCommandHandlerMetadata> = [];
  const claimedMethods: Map<string, Set<string>> = new Map();

  let current: Nullable<object> = constructor;

  while (current) {
    for (const metadata of COMMAND_HANDLERS.get(current) ?? []) {
      const claimed: Set<string> = claimedMethods.get(metadata.descriptor.id) ?? new Set();

      // A subclass redeclaring an inherited handler resolves to one method on the instance, so registering both
      // entries would put the same function on the stack twice and leave one behind when the first is disposed.
      if (!claimed.has(metadata.methodName)) {
        claimed.add(metadata.methodName);
        claimedMethods.set(metadata.descriptor.id, claimed);
        collected.push(metadata);
      }
    }

    current = Object.getPrototypeOf(current) as Nullable<object>;
  }

  return collected;
}

/**
 * Handlers declared by a provisioned instance's class.
 *
 * @param instance - Service instance being wired.
 * @returns Every command handler that instance declares.
 */
export function collectKeybindCommandHandlers(instance: object): ReadonlyArray<IKeybindCommandHandlerMetadata> {
  return collectFromConstructor(instance.constructor);
}

/**
 * Whether a binding token declares any command handler.
 *
 * @param token - Binding token to inspect.
 * @returns Whether that token declares commands.
 */
export function hasKeybindCommandHandlers(token: unknown): boolean {
  if (typeof token !== "function") {
    return false;
  }

  // Asked for every binding token of every container provision, so it walks the chain rather than collecting it.
  for (let current: Nullable<object> = token; current; current = Object.getPrototypeOf(current) as Nullable<object>) {
    if (COMMAND_HANDLERS.has(current)) {
      return true;
    }
  }

  return false;
}
