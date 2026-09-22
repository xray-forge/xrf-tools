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
