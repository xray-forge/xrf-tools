import { Loop } from "three/tsl";
import { Node } from "three/webgpu";

/** A loop's bounds, as three's `Loop` takes them, with the name its counter is declared under. */
export interface INamedLoop<T extends "int" | "uint"> {
  start: Node<T>;
  end: Node<T>;
  type: T;
  /** `<` unless said otherwise. */
  condition?: "<" | "<=";
  /** Unique within any loop it nests in. */
  name: string;
}

/**
 * Three's `Loop` with its counter named: three names every loop's counter `i` unless told otherwise, so a loop nested
 * in another shadows the outer counter and the body reads the inner one under both names. Three takes the name; its
 * typings do not say so.
 *
 * @param loop - The bounds and the counter's name.
 * @param body - What each turn does, given the counter.
 */
export function loopNamed<T extends "int" | "uint">(loop: INamedLoop<T>, body: (counter: Node<T>) => void): void {
  (Loop as unknown as (params: object, callback: (inputs: Record<string, Node<T>>) => void) => void)(
    { condition: "<", ...loop },
    (inputs: Record<string, Node<T>>) => body(inputs[loop.name])
  );
}
