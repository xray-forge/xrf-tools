import { Maybe } from "@xrf/types";
import { Node } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { toDeferredSurfaceShader } from "#/material/deferred-surface.tsl";
import { toForwardSurfaceShader } from "#/material/forward-surface.tsl";
import { toPickSurfaceShader } from "#/material/pick-surface.tsl";
import { toShadowSurfaceShader } from "#/material/shadow-surface.tsl";
import { ISurfaceInputs } from "#/material/surface-inputs";
import { toSurfaceInputs } from "#/material/surface-inputs.tsl";
import { ISurfaceShader } from "#/material/surface-shader";
import { ESurfaceSlot } from "#/material/surface-slot";
import { SurfaceSlotNodes } from "#/material/surface-slot-nodes";
import { toTabledSurfaceInputs } from "#/material/surface-table-inputs.tsl";
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
  [ERendererPass.WALLMARK]: toWallmarkSurfaceShader,
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
  /** Every shared slot sampler its shaders built. */
  public readonly nodes: SurfaceSlotNodes = new SurfaceSlotNodes();

  private readonly uniforms: RendererUniforms;
  private readonly inputs: ISurfaceInputs;
  private readonly shaders: Map<string, ISurfaceShader> = new Map();
  private readonly tabled: Map<string, ISurfaceShader> = new Map();
  private readonly tabledShadows: Map<string, ISurfaceShader> = new Map();
  private readonly tabledPicks: Map<string, ISurfaceShader> = new Map();
  private cutOutShadow: Maybe<ISurfaceShader>;
  private opaqueShadow: Maybe<ISurfaceShader>;
  private cutOutPick: Maybe<ISurfaceShader>;
  private wholePick: Maybe<ISurfaceShader>;

  /**
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(uniforms: RendererUniforms) {
    this.uniforms = uniforms;
    this.inputs = toSurfaceInputs(uniforms.settings.textureBias, this.nodes);
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
   * @param variant - A surface's variant.
   * @param arrayed - The slots a static batch's shared material samples from arrays.
   * @returns The shader every surface of the variant shares in a static batch, reading its numbers and array layers from
   *   the surface table: built the first time the pair is asked for.
   */
  public getTabled(variant: ISurfaceVariant, arrayed: ReadonlyArray<ESurfaceSlot>): ISurfaceShader {
    const key: string = `${toSurfaceVariantKey(variant)}|${arrayed.join(",")}`;
    let shader: Maybe<ISurfaceShader> = this.tabled.get(key);

    if (!shader) {
      const inputs: ISurfaceInputs = toTabledSurfaceInputs(
        this.uniforms.settings.textureBias,
        this.uniforms.surfaceTable,
        this.uniforms.staticDraws,
        arrayed,
        this.nodes
      );

      shader = SURFACE_SHADERS[variant.pass](variant, inputs, this.uniforms);
      this.tabled.set(key, shader);
    }

    return shader;
  }

  /**
   * @param arrayed - The slots a static batch's shared material samples from arrays, the base among them or not.
   * @returns What draws a cut-out caster sharing a static batch into a shadow map, its cut read from the surface table.
   */
  public getTabledShadow(arrayed: ReadonlyArray<ESurfaceSlot>): ISurfaceShader {
    const key: string = arrayed.join(",");
    let shader: Maybe<ISurfaceShader> = this.tabledShadows.get(key);

    if (!shader) {
      shader = toShadowSurfaceShader(
        toTabledSurfaceInputs(null, this.uniforms.surfaceTable, this.uniforms.staticDraws, arrayed, this.nodes)
      );
      this.tabledShadows.set(key, shader);
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

  /**
   * @param arrayed - The slots a static batch's shared material samples from arrays, the base among them or not.
   * @returns What draws a cut-out surface sharing a static batch into a pick, its cut read from the surface table.
   */
  public getTabledPick(arrayed: ReadonlyArray<ESurfaceSlot>): ISurfaceShader {
    const key: string = arrayed.join(",");
    let shader: Maybe<ISurfaceShader> = this.tabledPicks.get(key);

    if (!shader) {
      shader = toPickSurfaceShader(
        toTabledSurfaceInputs(null, this.uniforms.surfaceTable, this.uniforms.staticDraws, arrayed, this.nodes)
      );
      this.tabledPicks.set(key, shader);
    }

    return shader;
  }

  /**
   * @param isCutOut - Whether the surface is cut out, rather than drawn whole.
   * @returns What draws it into a pick.
   */
  public getPick(isCutOut: boolean): ISurfaceShader {
    if (isCutOut) {
      this.cutOutPick ??= toPickSurfaceShader(this.inputs.unbiased());

      return this.cutOutPick;
    }

    this.wholePick ??= toPickSurfaceShader(null);

    return this.wholePick;
  }

  /** Lets go of every sampler its shaders built, and the shaders with them. */
  public dispose(): void {
    this.nodes.clear();
    this.shaders.clear();
    this.tabled.clear();
    this.tabledShadows.clear();
    this.tabledPicks.clear();
    this.cutOutShadow = undefined;
    this.opaqueShadow = undefined;
    this.cutOutPick = undefined;
    this.wholePick = undefined;
  }
}
