import { uniform } from "three/tsl";
import { Camera, Vector3 } from "three/webgpu";

import { TRendererColor } from "#/contract/renderer-lighting";
import { IBaseLightingConstants } from "#/lighting/base-lighting";

/**
 * The constants the base lighting passes read, as shader uniforms updated in place.
 */
export class BaseLightingUniforms {
  public readonly sunColor = uniform(new Vector3());
  public readonly sunSpecular = uniform(0);
  /** The direction sunlight travels, in view space: set per frame from the world direction. */
  public readonly sunDirectionView = uniform(new Vector3(0, -1, 0));
  public readonly ambient = uniform(new Vector3());
  public readonly environment = uniform(new Vector3());
  public readonly skyIrradiance = uniform(new Vector3());
  public readonly fogOffset = uniform(0);
  public readonly fogScale = uniform(0);
  public readonly fogColor = uniform(new Vector3());
  public readonly tonemapScale = uniform(1);

  private readonly sunDirectionWorld: Vector3 = new Vector3(0, -1, 0);

  /** The direction sunlight travels, normalised, in world space. */
  public get sunDirection(): Vector3 {
    return this.sunDirectionWorld;
  }

  /**
   * @param constants - What the lighting value comes to.
   */
  public apply(constants: IBaseLightingConstants): void {
    BaseLightingUniforms.setColor(this.sunColor.value, constants.sunColor);
    BaseLightingUniforms.setColor(this.ambient.value, constants.ambient);
    BaseLightingUniforms.setColor(this.environment.value, constants.environment);
    BaseLightingUniforms.setColor(this.skyIrradiance.value, constants.skyIrradiance);
    BaseLightingUniforms.setColor(this.fogColor.value, constants.fogColor);

    this.sunSpecular.value = constants.sunSpecular;
    this.fogOffset.value = constants.fogOffset;
    this.fogScale.value = constants.fogScale;
    this.sunDirectionWorld.set(...constants.sunDirection);
  }

  /**
   * Carries the sun into the camera's space, as `transform_dir` into `mView` does for the engine.
   *
   * @param camera - The camera about to draw, with its matrices current.
   */
  public follow(camera: Camera): void {
    this.sunDirectionView.value.copy(this.sunDirectionWorld).transformDirection(camera.matrixWorldInverse);
  }

  /** Colours are held as vectors: raw engine numbers, which no colour management may reinterpret. */
  private static setColor(target: Vector3, value: TRendererColor): void {
    target.set(value[0], value[1], value[2]);
  }
}
