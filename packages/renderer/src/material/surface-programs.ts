import { Maybe } from "@xrf/types";
import { Node } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { toDeferredSurfaceShader } from "#/material/deferred-surface.tsl";
import { toForwardSurfaceShader } from "#/material/forward-surface.tsl";
import { toShadowSurfaceShader } from "#/material/shadow-surface.tsl";
import { ISurfaceInputs } from "#/material/surface-inputs";
import { toSurfaceInputs } from "#/material/surface-inputs.tsl";
import { ISurfaceShader } from "#/material/surface-shader";
import { ISurfaceVariant, toSurfaceVariantKey } from "#/material/surface-variant";
import { toWallmarkSurfaceShader } from "#/material/wallmark-surface.tsl";
import { toWaterSurfaceShader } from "#/material/water-surface.tsl";
import { instancedPosition } from "#/shader/placement.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** How each pass shades the surfaces it draws. */
const SURFACE_SHADERS: Record<
  ERendererPass,
  (variant: ISurfaceVariant, inputs: ISurfaceInputs, uniforms: RendererUniforms) => ISurfaceShader
> = {
  [ERendererPass.DEFERRED]: toDeferredSurfaceShader,
  [ERendererPass.FORWARD]: toForwardSurfaceShader,
  [ERendererPass.WALLMARK]: (variant: ISurfaceVariant, inputs: ISurfaceInputs) =>
    toWallmarkSurfaceShader(variant, inputs),
  [ERendererPass.WATER]: toWaterSurfaceShader,
};

/**
 * One shader per surface variant, every material of the variant drawing with its nodes: three builds a node graph for
 * each graph it has not seen, a graph made per surface is one it has never seen, and a level's surfaces are a thousand
 * of a few dozen kinds.
 */
export class SurfacePrograms {
  /** Where every surface stands its vertices. */
  public readonly position: Node<"vec3"> = instancedPosition();

  private readonly uniforms: RendererUniforms;
  private readonly inputs: ISurfaceInputs;
  private readonly shaders: Map<string, ISurfaceShader> = new Map();
  private cutOutShadow: Maybe<ISurfaceShader>;
  private opaqueShadow: Maybe<ISurfaceShader>;

  /**
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(uniforms: RendererUniforms) {
    this.uniforms = uniforms;
    this.inputs = toSurfaceInputs(uniforms.settings.textureBias);
  }

  /**
   * @param variant - A surface's variant.
   * @returns Its shader, built the first time the variant is asked for.
   */
  public get(variant: ISurfaceVariant): ISurfaceShader {
    const key: string = toSurfaceVariantKey(variant);
    let shader: Maybe<ISurfaceShader> = this.shaders.get(key);

    if (!shader) {
      shader = SURFACE_SHADERS[variant.pass](variant, this.inputs, this.uniforms);
      this.shaders.set(key, shader);
    }

    return shader;
  }

  /**
   * @param isCutOut - Whether the caster is cut out, rather than any opaque one.
   * @returns What draws it into a shadow map.
   */
  public getShadow(isCutOut: boolean): ISurfaceShader {
    if (isCutOut) {
      this.cutOutShadow ??= toShadowSurfaceShader(this.inputs.unbiased());

      return this.cutOutShadow;
    }

    this.opaqueShadow ??= toShadowSurfaceShader(null);

    return this.opaqueShadow;
  }
}
