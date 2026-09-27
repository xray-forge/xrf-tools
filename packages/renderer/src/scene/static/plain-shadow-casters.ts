import { Maybe } from "@xrf/types";
import { Mesh, Scene, Sphere, Vector4 } from "three/webgpu";

import { createSceneRoot } from "#/scene/object/scene-mesh";
import { isSphereInPlanes } from "#/scene/static/static-cell";

/** A twin casting in the shadow views, where it reaches, and whether its part draws anything at all. */
interface IPlainShadowCaster {
  readonly sphere: Sphere;
  isDrawing: boolean;
}

/**
 * The twins of the parts drawn plainly that cast: placed and skinned by three as their parts are, and shown to a shadow
 * view only where they reach into it, since three culls none of the scene's own meshes.
 */
export class PlainShadowCasters {
  /** What a shadow view draws of them, once shown the ones it holds. */
  public readonly scene: Scene = createSceneRoot();

  private readonly casters: Map<Mesh, IPlainShadowCaster> = new Map();

  /**
   * @param mesh - A twin casting from now on.
   * @param sphere - Where it reaches, the caller's own, kept up to date as the part moves.
   */
  public put(mesh: Mesh, sphere: Sphere): void {
    this.casters.set(mesh, { isDrawing: true, sphere });
    this.scene.add(mesh);
  }

  /**
   * @param mesh - A twin casting no more.
   */
  public release(mesh: Mesh): void {
    this.casters.delete(mesh);
    mesh.removeFromParent();
  }

  /**
   * @param mesh - A twin.
   * @param isDrawing - Whether its part draws anything, which a narrowing to nothing does not.
   */
  public setDrawing(mesh: Mesh, isDrawing: boolean): void {
    const caster: Maybe<IPlainShadowCaster> = this.casters.get(mesh);

    if (caster) {
      caster.isDrawing = isDrawing;
    }
  }

  /**
   * @param planes - A shadow view's planes.
   * @returns Whether it holds any twin, each shown or hidden by whether it reaches in.
   */
  public show(planes: ReadonlyArray<Vector4>): boolean {
    let isAny: boolean = false;

    this.casters.forEach((caster: IPlainShadowCaster, mesh: Mesh) => {
      mesh.visible = caster.isDrawing && isSphereInPlanes(caster.sphere, planes);
      isAny ||= mesh.visible;
    });

    return isAny;
  }
}
