import { IRendererLightAnimator } from "#/contract/scene/renderer-lights";

/**
 * `CLAItem::CalculateRGB`: the frame a time falls on, the animation looping over its frames, and the colour there,
 * each channel in `[0, 255]`.
 *
 * @param animator - The animation.
 * @param time - Seconds, as the engine's global time counts them.
 * @param out - Where the colour is written.
 * @returns The colour.
 */
export function toAnimatedColor(animator: IRendererLightAnimator, time: number, out: Array<number>): Array<number> {
  const length: number = animator.frameCount / animator.fps;
  const frame: number = length > 0 ? Math.floor((time % length) * animator.fps) : 0;

  return toInterpolatedColor(animator, frame, out);
}

/**
 * `CLAItem::InterpolateRGB`: a key's own colour on its frame, the last key's past the last, and between two keys the
 * two blended by how far the frame stands between them.
 *
 * @param animator - The animation.
 * @param frame - The frame, from zero.
 * @param out - Where the colour is written.
 * @returns The colour.
 */
export function toInterpolatedColor(
  animator: IRendererLightAnimator,
  frame: number,
  out: Array<number>
): Array<number> {
  const { frames, colors } = animator;
  const next: number = frames.findIndex((keyFrame: number) => keyFrame > frame);

  if (colors.length === 0) {
    out[0] = out[1] = out[2] = 0;

    return out;
  }

  // Past the last key, or on the first one with nothing before it to blend from.
  if (next <= 0) {
    const color = colors[next === 0 ? 0 : colors.length - 1];

    out[0] = color[0];
    out[1] = color[1];
    out[2] = color[2];

    return out;
  }

  const [from, to] = [colors[next - 1], colors[next]];
  const blend: number = (frame - frames[next - 1]) / (frames[next] - frames[next - 1]);

  for (let channel: number = 0; channel < 3; channel += 1) {
    out[channel] = from[channel] + (to[channel] - from[channel]) * blend;
  }

  return out;
}
