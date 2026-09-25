import {
  clamp,
  float,
  Fn,
  int,
  ivec2,
  max,
  min,
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
 * FSR 1's RCAS (`FsrRcasF`, AMD, MIT): each pixel sharpened by a lobe over its four neighbours, as strong as its
 * neighbourhood leaves room for without clipping, so an edge is crisped and a flat area left alone.
 *
 * @param frame - The resolved frame, colour and coverage, in the display's range.
 * @param strength - `exp2(-stops)`: one sharpens most, towards zero least.
 * @returns The sharpened colour, the frame's coverage kept.
 */
export function toSharpened(frame: Texture, strength: UniformNode<"float", number>): Node<"vec4"> {
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
    const hitMin: Node<"vec3"> = min(least, e).div(max(most.mul(4), vec3(1e-5)));
    const hitMax: Node<"vec3"> = float(1)
      .sub(max(most, e))
      .div(min(least.mul(4).sub(4), vec3(-1e-5)));
    const lobes: Node<"vec3"> = max(hitMin.negate(), hitMax);
    const lobe: Node<"float"> = max(float(-RCAS_LIMIT), min(max(lobes.x, max(lobes.y, lobes.z)), float(0))).mul(
      strength
    );
    const sharpened: Node<"vec3"> = b.add(d).add(f).add(h).mul(lobe).add(e).div(lobe.mul(4).add(1));

    return vec4(sharpened, centre.w);
  })();
}
