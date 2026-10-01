import { Nullable } from "@xrf/types";
import {
  BufferGeometry,
  Float32BufferAttribute,
  Mesh,
  NodeMaterial,
  Object3D,
  OrthographicCamera,
  RenderTarget,
  WebGPURenderer,
} from "three/webgpu";

import { compileInto } from "#/pass/compile-into";
import { toFullScreenVertex } from "#/pass/full-screen-draw.tsl";

/**
 * One full screen material drawn into one target, as three's `QuadMesh` draws it: a triangle over the screen, its
 * corners placed from the vertex index. Its material is its own and so is its pipeline, built for the target's
 * attachments, which the compile lane builds off the frame before the frame draws it. A pass names its draws as it holds
 * them, the same draw for as long as its material stands; one let go while it compiles goes once its compile ends,
 * since three is still building it until then.
 */
export class FullScreenDraw {
  public readonly material: NodeMaterial;
  /** Where it draws: the canvas when null. */
  public readonly target: Nullable<RenderTarget>;

  private readonly mesh: Mesh<BufferGeometry, NodeMaterial>;
  private readonly camera: OrthographicCamera = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
  /** The compile in flight, if any. */
  private compiling: Nullable<Promise<void>> = null;

  /**
   * @param material - What each pixel comes to, the draw's from now on: its vertex is set to the triangle's.
   * @param target - Where it draws: the canvas when null.
   */
  public constructor(material: NodeMaterial, target: Nullable<RenderTarget>) {
    const geometry: BufferGeometry = new BufferGeometry();

    // Three's `QuadGeometry`: the corners the vertex places, and the `uv` a material may read there.
    geometry.setAttribute("position", new Float32BufferAttribute([-1, 3, 0, -1, -1, 0, 3, -1, 0], 3));
    geometry.setAttribute("uv", new Float32BufferAttribute([0, -1, 0, 1, 2, 1], 2));
    material.vertexNode = toFullScreenVertex();
    this.material = material;
    this.target = target;
    this.mesh = new Mesh(geometry, material);
    this.mesh.frustumCulled = false;
  }

  /** The triangle as an object, for a pass drawing it among others in one call: its corners ignore the camera. */
  public get object(): Object3D {
    return this.mesh;
  }

  /**
   * @param renderer - The renderer drawing.
   * @param target - Where it draws: its own target, or another of the same attachments, whose pipeline is the same.
   */
  public render(renderer: WebGPURenderer, target: Nullable<RenderTarget> = this.target): void {
    renderer.setRenderTarget(target);
    renderer.render(this.mesh, this.camera);
  }

  /**
   * @param renderer - The renderer drawing.
   * @returns Settles once its pipeline is built.
   */
  public compile(renderer: WebGPURenderer): Promise<void> {
    const compiling: Promise<void> = compileInto(renderer, this.target, this.mesh, this.camera);
    const settle = (): void => {
      if (this.compiling === compiling) {
        this.compiling = null;
      }
    };

    this.compiling = compiling;
    compiling.then(settle, settle);

    return compiling;
  }

  public dispose(): void {
    const release = (): void => {
      this.material.dispose();
      this.mesh.geometry.dispose();
    };

    if (this.compiling) {
      this.compiling.then(release, release);
    } else {
      release();
    }
  }
}
