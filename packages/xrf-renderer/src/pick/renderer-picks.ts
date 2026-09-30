import { Nullable } from "@xrf/types";
import {
  Color,
  FloatType,
  Material,
  Mesh,
  Object3D,
  PerspectiveCamera,
  Ray,
  RenderTarget,
  Scene,
  TypedArray,
  Vector3,
  WebGPURenderer,
} from "three/webgpu";

import { IRendererViewPoint } from "#/contract/renderer-view-point";
import { IRendererHit } from "#/contract/scene/renderer-hit";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { drawUnsorted } from "#/pass/unsorted-draw";
import { IPickView } from "#/pick/pick-view";
import { IPickTexel, toPickTexel } from "#/scene/object/pick-texel";
import { RendererScene } from "#/scene/renderer-scene";

/** A pick waiting for a frame to answer it. */
interface IPendingPick {
  id: number;
  point: IRendererViewPoint;
}

/** Hands what a pick hit back, or nothing. */
type TPickReply = (id: number, hit: Nullable<IRendererHit>) => void;

/**
 * The picks asked for, each drawn after a frame into one texel under its point, every surface by its pick twin, and
 * read back to the draw it names. Nothing is drawn for a pick until one is asked for, and nothing is kept for one
 * between them but the texel.
 */
export class RendererPicks {
  private readonly scene: RendererScene;
  private readonly reply: TPickReply;
  /** The texel every pick draws into in turn, and its depth: a read is queued before the next pick draws. */
  private readonly target: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true, type: FloatType });
  /** The frame's camera, narrowed to the one texel under a point. */
  private readonly camera: PerspectiveCamera = new PerspectiveCamera();
  private readonly clearColor: Color = new Color();
  private pending: Array<IPendingPick> = [];
  private isDisposed: boolean = false;

  public constructor(scene: RendererScene, reply: TPickReply) {
    this.scene = scene;
    this.reply = reply;
  }

  public get hasPending(): boolean {
    return this.pending.length > 0;
  }

  /**
   * @param id - What the answer is sent under.
   * @param point - Where in the view to pick.
   */
  public push(id: number, point: IRendererViewPoint): void {
    this.pending.push({ id, point });
  }

  /**
   * Draws every pick waiting at the frame just drawn and reads it back.
   *
   * @param renderer - The renderer drawing.
   * @param view - The view the frame was drawn at, or null where there is none, which hits nothing.
   */
  public answer(renderer: WebGPURenderer, view: Nullable<IPickView>): void {
    const picks: ReadonlyArray<IPendingPick> = this.pending;

    this.pending = [];

    for (const { id, point } of picks) {
      if (view && RendererPicks.isInside(point, view)) {
        this.pick(renderer, id, point, view);
      } else {
        this.reply(id, null);
      }
    }
  }

  /** Drops every pick waiting and every read in flight unanswered: the renderer is gone, which answers them. */
  public dispose(): void {
    this.isDisposed = true;
    this.pending = [];
    this.target.dispose();
  }

  private pick(renderer: WebGPURenderer, id: number, point: IRendererViewPoint, view: IPickView): void {
    const { width, height } = view.size;
    // Taken now: by the time the texel is read the camera has moved on.
    const ray: Ray = RendererPicks.toRay(point, view);

    this.camera.copy(view.camera);
    // The texel centred on the point, which is where the ray passes.
    this.camera.setViewOffset(width, height, point.x - 0.5, point.y - 0.5, 1, 1);
    this.draw(renderer);

    renderer
      .readRenderTargetPixelsAsync(this.target, 0, 0, 1, 1)
      .then((pixels: TypedArray) => {
        if (this.isDisposed) {
          return;
        }

        const texel: Nullable<IPickTexel> = toPickTexel(pixels);
        const hit: Nullable<Omit<IRendererHit, "point">> = texel ? this.scene.findHit(texel) : null;
        const at: Vector3 = texel ? ray.at(texel.distance, new Vector3()) : ray.origin;

        this.reply(id, hit ? { ...hit, point: [at.x, at.y, at.z] } : null);
      })
      .catch(() => {
        if (!this.isDisposed) {
          this.reply(id, null);
        }
      });
  }

  /**
   * Draws every surface a point can land on into the texel by its pick twin, hiding whatever has none, and leaves
   * every mesh and the renderer as it found them.
   */
  private draw(renderer: WebGPURenderer): void {
    const restores: Array<() => void> = [];
    const target: Nullable<RenderTarget> = renderer.getRenderTarget();
    const alpha: number = renderer.getClearAlpha();

    renderer.getClearColor(this.clearColor);

    for (const scene of this.scene.pickedScenes) {
      scene.traverse((object: Object3D) => {
        const restore: Nullable<() => void> = RendererPicks.toPickDrawn(object);

        if (restore) {
          restores.push(restore);
        }
      });
    }

    try {
      renderer.setRenderTarget(this.target);
      renderer.setClearColor(0x000000, 0);
      renderer.clear();
      drawUnsorted(renderer, () =>
        this.scene.pickedScenes.forEach((scene: Scene) => renderer.render(scene, this.camera))
      );
    } finally {
      restores.forEach((restore: () => void) => restore());
      renderer.setClearColor(this.clearColor, alpha);
      renderer.setRenderTarget(target);
    }
  }

  /**
   * Has a mesh draw by its material's pick twin, or not at all where the material has none.
   *
   * @returns What puts it back, or null for an object left as it is.
   */
  private static toPickDrawn(object: Object3D): Nullable<() => void> {
    const mesh: Mesh = object as Mesh;

    if (!mesh.isMesh) {
      return null;
    }

    const material: Material | Array<Material> = mesh.material;
    const pick: Nullable<SurfaceNodeMaterial> = material instanceof SurfaceNodeMaterial ? material.pick : null;

    if (pick) {
      mesh.material = pick;

      return () => (mesh.material = material);
    }

    if (mesh.visible) {
      mesh.visible = false;

      return () => (mesh.visible = true);
    }

    return null;
  }

  private static isInside(point: IRendererViewPoint, view: IPickView): boolean {
    const { width, height } = view.size;

    return width >= 1 && height >= 1 && point.x >= 0 && point.y >= 0 && point.x <= width && point.y <= height;
  }

  /** The ray from the camera through a point of the view, in renderer space. */
  private static toRay(point: IRendererViewPoint, view: IPickView): Ray {
    const { camera, size } = view;
    const origin: Vector3 = camera.getWorldPosition(new Vector3());
    const through: Vector3 = new Vector3(
      (point.x / size.width) * 2 - 1,
      1 - (point.y / size.height) * 2,
      0.5
    ).unproject(camera);

    return new Ray(origin, through.sub(origin).normalize());
  }
}
