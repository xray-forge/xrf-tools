import {
  Discard,
  dot,
  float,
  Fn,
  getViewPosition,
  If,
  length,
  mix,
  normalize,
  reflect,
  saturate,
  screenUV,
  texture,
  texture3D,
  vec3,
  vec4,
} from "three/tsl";
import { Color, Data3DTexture, LinearSRGBColorSpace, NodeMaterial, QuadMesh } from "three/webgpu";

import { BaseLightingUniforms } from "#/graph/base-lighting-uniforms";
import { CameraUniforms } from "#/graph/camera-uniforms";
import { decodeOctahedral } from "#/graph/octahedral-normal.tsl";
import { IRendererFrame } from "#/graph/renderer-frame";
import { RendererTargets } from "#/graph/renderer-targets";
import { toneMapReinhardNode } from "#/graph/tonemap.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * `combine_1` and the tonemap of `combine_2`: the hemisphere model, the lights, fog, and the engine's Reinhard curve.
 */
export class CombinePass implements IRendererPass {
  public readonly name: string = "combine";

  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;
  private readonly backdrop: Color = new Color();

  public constructor(
    targets: RendererTargets,
    camera: CameraUniforms,
    lighting: BaseLightingUniforms,
    lut: Data3DTexture
  ) {
    const fragment = Fn(() => {
      const depth = texture(targets.depth, screenUV).x;

      // Where nothing was drawn the backdrop the target was cleared to shows.
      If(depth.greaterThanEqual(1), () => {
        Discard();
      });

      const position = getViewPosition(screenUV, depth, camera.projectionInverse);
      const normal = decodeOctahedral(texture(targets.normal, screenUV).xy);
      const albedo = texture(targets.albedo, screenUV);
      const surface = texture(targets.surface, screenUV);
      const light = texture(targets.light.texture, screenUV);

      // `hmodel`: the hemisphere looked up by occlusion and by how far the reflection turns from the view.
      const normalWorld = normalize(camera.viewToWorld.mul(vec4(normal, 0)).xyz);
      const toPointWorld = normalize(camera.viewToWorld.mul(vec4(position, 0)).xyz);
      const hemisphereSpecular = float(0.5).add(dot(reflect(toPointWorld, normalWorld), toPointWorld).mul(0.5));
      const hemisphere = texture3D(lut, vec3(surface.x, hemisphereSpecular, surface.z));
      // The irradiance cube stands in as one colour until weather supplies the cube itself.
      const environment = lighting.environment.mul(lighting.skyIrradiance);
      const environmentSquared = environment.mul(environment);
      const hemisphereDiffuse = environmentSquared.mul(hemisphere.x).add(lighting.ambient);
      const hemisphereGloss = environmentSquared.mul(hemisphere.y).mul(albedo.w);

      const color = albedo.xyz.mul(light.xyz.add(hemisphereDiffuse)).add(albedo.w.mul(light.w)).add(hemisphereGloss);
      const fog = saturate(length(position).mul(lighting.fogScale).add(lighting.fogOffset));

      return vec4(toneMapReinhardNode(mix(color, lighting.fogColor, fog), lighting.tonemapScale), 1);
    })();

    this.material = createQuadMaterial(fragment);
    this.quad = new QuadMesh(this.material);
  }

  public render({ renderer, targets, settings }: IRendererFrame): void {
    // The hex is bytes the page shows, so it reaches the canvas as written rather than decoded from srgb.
    this.backdrop.setHex(settings.backdrop ?? 0, LinearSRGBColorSpace);
    renderer.setClearColor(this.backdrop, settings.backdrop === null ? 0 : 1);
    renderer.setRenderTarget(targets.scene);
    renderer.clear(true, false, false);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
  }
}
