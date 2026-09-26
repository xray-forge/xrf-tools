import { uniform } from "three/tsl";
import { PerspectiveCamera, Vector3 } from "three/webgpu";

import { IRendererLodSettings } from "#/contract/renderer-features";

/** `EPS_S`, what `g_fSCREEN`'s LOD scale is offset by so it is never zero (`xrCore/math_constants.h`). */
const SCREEN_EPSILON: number = 0.0000001;

/** The field of view the engine's thresholds are measured against, which a wider one shrinks every clump by. */
const REFERENCE_FIELD_OF_VIEW: number = 90;

/**
 * What the LOD cull decides a clump of trees by: where the camera stands, and the engine's thresholds on a clump's
 * screen area, which scale with the drawing's size and field of view (`r2_R_calculate.cpp`).
 */
export class LodUniforms {
  public readonly camera = uniform(new Vector3());
  /** `r_ssaLOD_A`: below it the impostor draws. */
  public readonly lodA = uniform(0);
  /** `r_ssaLOD_B`: above it the trees draw. */
  public readonly lodB = uniform(0);
  /** `r_ssaDISCARD`: below it neither draws. */
  public readonly discard = uniform(0);
  /** One where impostors draw at all, zero where every clump draws its trees. */
  public readonly isEnabled = uniform(1);
  /** `r_ssaGLOD_start`: above it a progressive mesh draws its whole detail. */
  public readonly glodStart = uniform(0);
  /** `r_ssaGLOD_end`: below it a progressive mesh draws its coarsest window. */
  public readonly glodEnd = uniform(0);

  /**
   * @param settings - The consumer's LOD settings.
   * @param width - The drawing's width, in pixels.
   * @param height - Its height.
   * @param camera - The camera drawing it, with its matrices current.
   * @returns Whether anything the cull reads changed.
   */
  public take(settings: IRendererLodSettings, width: number, height: number, camera: PerspectiveCamera): boolean {
    const fieldOfView: number = REFERENCE_FIELD_OF_VIEW / camera.fov;
    const screen: number = width * height * fieldOfView * fieldOfView * (SCREEN_EPSILON + settings.geometryLod);
    // Every threshold set, whichever changed: none may be skipped once one has.
    let isChanged: boolean = setChanged(this.lodA, (settings.ssaA / 3) ** 2 / screen);

    isChanged = setChanged(this.lodB, (settings.ssaB / 3) ** 2 / screen) || isChanged;
    isChanged = setChanged(this.discard, settings.ssaDiscard ** 2 / screen) || isChanged;
    isChanged = setChanged(this.isEnabled, settings.isImpostors ? 1 : 0) || isChanged;
    isChanged = setChanged(this.glodStart, (settings.ssaGlodStart / 3) ** 2 / screen) || isChanged;
    isChanged = setChanged(this.glodEnd, (settings.ssaGlodEnd / 3) ** 2 / screen) || isChanged;

    camera.getWorldPosition(this.camera.value);

    return isChanged;
  }
}

/**
 * @param uniform - A uniform.
 * @param uniform.value - What it holds.
 * @param value - What it holds from now on.
 * @returns Whether that is not what it held.
 */
function setChanged(uniform: { value: number }, value: number): boolean {
  const isChanged: boolean = uniform.value !== value;

  uniform.value = value;

  return isChanged;
}
