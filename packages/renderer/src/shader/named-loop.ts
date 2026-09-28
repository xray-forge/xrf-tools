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
