import {
  abs,
  clamp,
  float,
  Fn,
  int,
  ivec2,
  max,
  min,
  saturate,
  screenCoordinate,
  texture,
  textureLoad,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node, Texture, UniformNode } from "three/webgpu";

/** `FSR_RCAS_LIMIT`: the most negative lobe, so the sharpened pixel never rings past its neighbours. */
const RCAS_LIMIT: number = 0.25 - 1 / 16;

/**
 * `FsrRcasF` (FidelityFX, `ffx_fsr1.h`, MIT): each pixel sharpened by a lobe over its four neighbours, as strong as its
 * neighbourhood leaves room for without clipping, so an edge is crisped and a flat area left alone; with the denoise,
 * the lobe halves where the centre stands apart from its ring, so noise is not sharpened too.
 *
 * @param frame - The upscaled frame, colour and coverage, in the display's range.
 * @param strength - `exp2(-stops)`: one sharpens most, towards zero least.
 * @param isDenoised - Whether `FSR_RCAS_DENOISE` is on, as FSR 2's own RCAS has it.
 * @returns The sharpened colour, the frame's coverage kept.
 */
export function toSharpened(frame: Texture, strength: UniformNode<"float", number>, isDenoised: boolean): Node<"vec4"> {
  return Fn(() => {
    const last: Node<"vec2"> = vec2(texture(frame).size(int(0)) as Node<"uvec2">).sub(1);
    const at: Node<"vec2"> = screenCoordinate.xy.floor();

    function load(x: number, y: number): Node<"vec4"> {
      return textureLoad(frame, ivec2(clamp(at.add(vec2(x, y)), vec2(0), last)));
    }

    const centre: Node<"vec4"> = load(0, 0);
    const [b, d, f, h] = [load(0, -1), load(-1, 0), load(1, 0), load(0, 1)].map((it) => it.xyz);
    const e: Node<"vec3"> = centre.xyz;
    const least: Node<"vec3"> = min(min(b, d), min(f, h));
    const most: Node<"vec3"> = max(max(b, d), max(f, h));
    // How far each channel may go down before the darkest neighbour clips, and up before the brightest does.
    const hitMin: Node<"vec3"> = least.div(max(most.mul(4), vec3(1e-5)));
    const hitMax: Node<"vec3"> = float(1)
      .sub(most)
      .div(min(least.mul(4).sub(4), vec3(-1e-5)));
    const lobes: Node<"vec3"> = max(hitMin.negate(), hitMax);
    let lobe: Node<"float"> = max(float(-RCAS_LIMIT), min(max(lobes.x, max(lobes.y, lobes.z)), float(0))).mul(strength);

    if (isDenoised) {
      // Luma times two, and how far the centre stands from its ring's mean against the ring's whole range.
      function luma(it: Node<"vec3">): Node<"float"> {
        return it.z.mul(0.5).add(it.x.mul(0.5).add(it.y));
      }

      const [bL, dL, eL, fL, hL] = [b, d, e, f, h].map(luma);
      const range: Node<"float"> = max(max(max(bL, dL), max(eL, fL)), hL).sub(min(min(min(bL, dL), min(eL, fL)), hL));
      const noise: Node<"float"> = saturate(abs(bL.add(dL).add(fL).add(hL).mul(0.25).sub(eL)).div(max(range, 1e-5)));

      lobe = lobe.mul(noise.mul(-0.5).add(1));
    }

    const sharpened: Node<"vec3"> = b.add(d).add(f).add(h).mul(lobe).add(e).div(lobe.mul(4).add(1));

    return vec4(sharpened, centre.w);
  })();
}
