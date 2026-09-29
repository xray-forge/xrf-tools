import { renderGroup, uniform } from "three/tsl";
import { Camera, UniformNode, Vector3 } from "three/webgpu";

import { TRendererColor } from "#/contract/renderer-color";
import { IBaseLightingConstants } from "#/lighting/base-lighting-constants";

/**
 * The constants the base lighting passes read, as shader uniforms updated in place.
 */
export class BaseLightingUniforms {
  public readonly sunColor: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  public readonly sunSpecular: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** The direction sunlight travels, in view space: set per frame from the world direction. */
  public readonly sunDirectionView: UniformNode<"vec3", Vector3> = uniform(new Vector3(0, -1, 0)).setGroup(renderGroup);
  public readonly ambient: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  public readonly environment: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  public readonly skyIrradiance: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  public readonly fogOffset: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  public readonly fogScale: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  public readonly fogColor: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  /** One while there is fog, zero without. */
  public readonly fogged: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** One while surfaces are shaded as Anomaly's `hmodel` and `combine_1` shade them. */
  public readonly extendedShading: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** `rain_params.x`: how hard it rains. */
  public readonly rainDensity: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);

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
    this.fogged.value = constants.isFogged ? 1 : 0;
    this.extendedShading.value = constants.isExtendedShading ? 1 : 0;
    this.rainDensity.value = constants.rainDensity;
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
