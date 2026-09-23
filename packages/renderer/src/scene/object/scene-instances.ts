import { BufferGeometry, InstancedBufferGeometry } from "three/webgpu";

import { IRendererInstances } from "#/contract/scene/renderer-object";
import { createInstancedGeometry } from "#/scene/geometry/instanced-geometry";

/**
 * What an object stands in many places with: its own geometry over the one put, adding the places as instanced
 * attributes, drawn by every pass.
 */
export class SceneInstances {
  /** The geometry put under the object's key, whose attributes the instanced one shares. */
  public readonly base: BufferGeometry;
  public readonly source: IRendererInstances;
  public readonly geometry: InstancedBufferGeometry;

  public constructor(base: BufferGeometry, source: IRendererInstances) {
    this.base = base;
    this.source = source;
    this.geometry = createInstancedGeometry(base, source);
  }

  /**
   * @param base - The geometry the object names now.
   * @param source - The places it names now.
   * @returns Whether these are still the places it stands in.
   */
  public isFor(base: BufferGeometry, source: IRendererInstances): boolean {
    return this.base === base && this.source === source;
  }

  public dispose(): void {
    this.geometry.dispose();
  }
}
