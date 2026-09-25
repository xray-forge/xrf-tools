import { Nullable } from "@xrf/types";
import { float, Fn, log, screenUV, select, texture, vec3, vec4 } from "three/tsl";
import { Node, RenderTarget, Texture } from "three/webgpu";

import { ERendererDebugView } from "#/contract/renderer-settings";
import { RendererTargets } from "#/pass/renderer-targets";
import { toUpsampledAmbientOcclusion } from "#/shader/ambient-occlusion.tsl";
import { IGBufferSample } from "#/shader/gbuffer-sample";
import { readGBuffer } from "#/shader/gbuffer.tsl";
import { CameraUniforms } from "#/uniforms/camera-uniforms";

/**
 * @param view - The picture wanted.
 * @param targets - The frame's targets.
 * @param camera - The drawing camera's uniforms.
 * @param frame - What the finished frame is read from.
 * @param ambientOcclusion - The screen's occlusion at half resolution, or none.
 * @returns The picture at every pixel: the finished frame, or one target shown raw.
 */
export function toPresentPassFragment(
  view: ERendererDebugView,
  targets: RendererTargets,
  camera: CameraUniforms,
  frame: RenderTarget,
  ambientOcclusion: Nullable<Texture>
): Node<"vec4"> {
  if (view === ERendererDebugView.FINAL) {
    return texture(frame.texture, screenUV);
  }

  return Fn(() => {
    const sample: IGBufferSample = readGBuffer(targets, camera);
    // A target shown raw is transparent where nothing was drawn, so it is never mistaken for a cleared value.
    const coverage: Node<"float"> = select(sample.depth.greaterThan(0), float(1), float(0));

    return vec4(toShownTarget(view, sample, targets, camera, ambientOcclusion), coverage);
  })();
}

/** One target as a colour. */
function toShownTarget(
  view: Exclude<ERendererDebugView, ERendererDebugView.FINAL>,
  sample: IGBufferSample,
  targets: RendererTargets,
  camera: CameraUniforms,
  ambientOcclusion: Nullable<Texture>
): Node<"vec3"> {
  switch (view) {
    case ERendererDebugView.ALBEDO:
      return sample.albedo;

    case ERendererDebugView.GLOSS:
      return vec3(sample.gloss);

    case ERendererDebugView.NORMAL:
      return sample.point.normal.mul(0.5).add(0.5);

    case ERendererDebugView.HEMI:
      return vec3(sample.hemi);

    case ERendererDebugView.SUN:
      return vec3(sample.sun);

    case ERendererDebugView.MATERIAL:
      return vec3(sample.point.slice);

    case ERendererDebugView.DEPTH:
      // Logarithmic, so a metre up close and a kilometre away both read.
      return vec3(log(sample.point.position.z.negate().add(1)).div(log(camera.far.add(1))));

    case ERendererDebugView.LIGHT:
      return texture(targets.light.texture, screenUV).xyz;

    case ERendererDebugView.AMBIENT_OCCLUSION:
      return vec3(
        ambientOcclusion
          ? toUpsampledAmbientOcclusion(ambientOcclusion, sample.point.position.z.negate(), sample.depth.greaterThan(0))
          : float(1)
      );
  }
}
