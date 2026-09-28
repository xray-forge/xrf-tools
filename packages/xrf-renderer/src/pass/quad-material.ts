import { Node, NodeMaterial } from "three/webgpu";

/**
 * A full screen material: it writes colour only, so a quad never touches the depth a later pass tests against.
 *
 * @param fragment - What each pixel comes to.
 * @returns The material.
 */
export function createQuadMaterial(fragment: Node): NodeMaterial {
  const material: NodeMaterial = new NodeMaterial();

  material.fragmentNode = fragment;
  material.depthTest = false;
  material.depthWrite = false;

  return material;
}

/**
 * A full screen material that writes a depth of its own at every pixel, whatever stood there: the test is left off,
 * since three turns `AlwaysDepth` into `NeverDepth` for a reversed depth buffer.
 *
 * @param fragment - What each pixel comes to.
 * @param depth - The depth each pixel writes.
 * @returns The material.
 */
export function createDepthWritingQuadMaterial(fragment: Node, depth: Node<"float">): NodeMaterial {
  const material: NodeMaterial = createQuadMaterial(fragment);

  material.depthNode = depth;
  material.depthWrite = true;

  return material;
}
