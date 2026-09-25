import { TRendererColor, TRendererVector } from "#/contract/renderer-lighting";

/**
 * The shape a light reaches out in.
 */
export enum ERendererLightKind {
  /** Every way around it, to its range. */
  POINT = "point",
  /** Along its direction, within its cone, through its projector. */
  SPOT = "spot",
}

/**
 * One local light, as the engine hands it to its shaders: accumulated after the sun, `Ldynamic_color` times
 * `plight_local`'s falloff to its range, a spot through its projector.
 */
export interface IRendererLight {
  kind: ERendererLightKind;
  position: TRendererVector;
  /** Where a spot points. */
  direction: TRendererVector;
  /** What turns a spot's projector about its direction. */
  right: TRendererVector;
  /** Raw, as the engine sets it: a lamp's colour times its brightness. */
  color: TRendererColor;
  range: number;
  /** A spot's whole cone, in radians. */
  cone: number;
  /** Where a spot's projection starts. */
  near: number;
  /** A spot's projector: the key its texture was put under. Without one it lights its whole cone white. */
  projector?: string;
  /** The animation replacing its colour, by its index among the lights' animators. */
  animator?: number;
  /** What an animated colour, each channel in `[0, 255]`, is multiplied by. */
  animatorScale: number;
  /** Whether it casts shadows, which also has it fade and drop out with distance as the engine's shadowed lights do. */
  isShadowed: boolean;
  /** Whether it is one of the level file's own lights, which the engine draws only with `r2_allow_r1_lights`. */
  isLevel: boolean;
}

/**
 * A colour animation (`CLAItem`): keys a frame apart at its rate, the colour between two keys blended by the frame.
 */
export interface IRendererLightAnimator {
  fps: number;
  frameCount: number;
  /** Each key's frame, ascending, the first at zero. */
  frames: ReadonlyArray<number>;
  /** Each key's colour, each channel in `[0, 255]`. */
  colors: ReadonlyArray<TRendererColor>;
}

/**
 * The local lights of a scene, and the animations they name.
 */
export interface IRendererLights {
  lights: ReadonlyArray<IRendererLight>;
  animators: ReadonlyArray<IRendererLightAnimator>;
}
