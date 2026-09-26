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
 * What every local light is, as the engine hands it to its shaders: accumulated after the sun, `Ldynamic_color` times
 * `plight_local`'s falloff to its range.
 */
export interface IRendererLightBase {
  position: TRendererVector;
  /** Raw, as the engine sets it: a lamp's colour times its brightness. */
  color: TRendererColor;
  range: number;
  /** How far the range strays each frame, either way, at random: a zone's flicker. None where left out. */
  rangeJitter?: number;
  /** Where its shadow faces' projection starts; the engine's default where zero. */
  near: number;
  /** The animation replacing its colour, by its index among the lights' animators. */
  animator?: number;
  /** What an animated colour, each channel in `[0, 255]`, is multiplied by. */
  animatorScale: number;
  /** Whether it casts shadows, which also has it fade and drop out with distance as the engine's shadowed lights do. */
  isShadowed: boolean;
  /** Whether it is one of the level file's own lights, which the engine draws only with `r2_allow_r1_lights`. */
  isLevel: boolean;
}

/** A light reaching every way around it. */
export interface IRendererPointLight extends IRendererLightBase {
  kind: ERendererLightKind.POINT;
}

/** A light reaching along its direction, within its cone, through its projector. */
export interface IRendererSpotLight extends IRendererLightBase {
  kind: ERendererLightKind.SPOT;
  direction: TRendererVector;
  /** What turns its projector about its direction. */
  right: TRendererVector;
  /** Its whole cone, in radians. */
  cone: number;
  /** Its projector: the key its texture was put under. Without one it lights its whole cone white. */
  projector?: string;
}

/** One local light, of either kind. */
export type TRendererLight = IRendererPointLight | IRendererSpotLight;

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
  lights: ReadonlyArray<TRendererLight>;
  animators: ReadonlyArray<IRendererLightAnimator>;
}
