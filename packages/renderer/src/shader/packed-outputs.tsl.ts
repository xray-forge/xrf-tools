import { int, mat4, outputStruct, vec4 } from "three/tsl";
import { Node } from "three/webgpu";

/**
 * Several targets' outputs from one `Fn`: an `outputStruct` built inside a `Fn` compiles to members that do not exist,
 * so the `Fn` returns them as a matrix's columns and `unpackOutputs` builds the struct outside it.
 *
 * @param outputs - Up to four outputs.
 * @returns The packed matrix, for the `Fn` to return.
 */
export function packOutputs(...outputs: Array<Node<"vec4">>): Node<"mat4"> {
  const columns: Array<Node<"vec4">> = [...outputs, vec4(0), vec4(0), vec4(0)].slice(0, 4);

  return mat4(columns[0], columns[1], columns[2], columns[3]);
}

/**
 * @param packed - What `packOutputs` packed, as the `Fn` returned it.
 * @param count - How many outputs it packed.
 * @returns The outputs, one a target.
 */
export function unpackOutputs(packed: Node<"mat4">, count: number): Node {
  const columns = packed as unknown as { element(index: Node<"int">): Node<"vec4"> };

  return outputStruct(...Array.from({ length: count }, (_: unknown, index: number) => columns.element(int(index))));
}
