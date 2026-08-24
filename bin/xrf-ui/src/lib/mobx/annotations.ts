import { Nullable } from "@/lib/types/general";

/**
 * Description of the MobX symbol holding the annotations a decorator recorded on a prototype.
 */
const MOBX_STORED_ANNOTATIONS: string = "mobx-stored-annotations";

/**
 * Reports whether an instance has MobX decorated members, and so anything for `makeObservable` to apply.
 *
 * Asked so plain container services - an event bus, a command bus - are left without an observable administration
 * they would never read.
 *
 * @param instance - Instance to inspect.
 * @returns Whether a prototype in its chain carries recorded MobX annotations.
 */
export function hasObservableMembers(instance: object): boolean {
  let prototype: Nullable<object> = Object.getPrototypeOf(instance);

  while (prototype && prototype !== Object.prototype) {
    const isAnnotated: boolean = Object.getOwnPropertySymbols(prototype).some(
      (it) => it.description === MOBX_STORED_ANNOTATIONS
    );

    if (isAnnotated) {
      return true;
    }

    prototype = Object.getPrototypeOf(prototype);
  }

  return false;
}
