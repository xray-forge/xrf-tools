import { setCurrentStack } from "three/tsl";
import { Node, StackNode } from "three/webgpu";

/** What a `Fn` called in a graph is to three: the call, holding the function whose body builds its nodes. */
interface IFunctionCall {
  isShaderCallNodeInternal?: boolean;
  shaderNode: { jsFunc: () => unknown };
}

/**
 * Whether a shader's graph reads a node, through every child and through the body of every `Fn` it calls: a body builds
 * its nodes only as three builds the shader, so one taking nothing is run here over a stack of its own, as a build runs
 * it. A body taking inputs reads them, which are the call's children; one taking the builder asks it what is built.
 *
 * @param root - The shader's output.
 * @param target - The node looked for, a uniform say.
 * @returns Whether the output depends on it.
 */
export function isNodeReading(root: Node, target: Node): boolean {
  const seen: Set<number> = new Set();
  const pending: Array<Node> = [root];

  while (pending.length) {
    const node: Node = pending.pop() as Node;

    if (node.id === target.id) {
      return true;
    }

    if (seen.has(node.id)) {
      continue;
    }

    seen.add(node.id);
    pending.push(...node.getChildren());

    const call: IFunctionCall = node as unknown as IFunctionCall;

    if (call.isShaderCallNodeInternal && call.shaderNode.jsFunc.length === 0) {
      const stack: StackNode = new StackNode();

      setCurrentStack(stack);

      try {
        const output: unknown = call.shaderNode.jsFunc();

        pending.push(stack);

        if (output instanceof Node) {
          pending.push(output);
        }
      } finally {
        setCurrentStack(null);
      }
    }
  }

  return false;
}
