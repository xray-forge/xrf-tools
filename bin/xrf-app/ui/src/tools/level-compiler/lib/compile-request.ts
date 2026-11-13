import {
  AiMapCovers,
  ELevelCompileStep,
  LevelCompileRequest,
  LevelCompileStep,
  RayBackend,
  RayGpu,
} from "@xrf/ipc/types/xrf-level-compiler";
import { Nullable } from "@xrf/types";

/** What the form names for the compile after the build. */
export interface ICompileChoice {
  steps: ReadonlySet<LevelCompileStep>;
  /** The SDK's `editors` directory, whose `bin` holds xrLC. */
  sdk: Nullable<string>;
  /** The game data holding `gamemtl.xr` and `shaders_xrlc.xr`. */
  gamedata: Nullable<string>;
  /** The textures root, with a `.thm` for every texture `build.prj` names. */
  textures: Nullable<string>;
  /** What traces the rays: the details' shadow rays and the AI map's cover rays. */
  rayBackend: RayBackend;
  /** The GPU they trace on, the renderer's choice; null for the automatic one. */
  rayGpu: Nullable<RayGpu>;
  /** The covers the AI map gets: full ones traced against `build.cform`, or none. */
  aiCovers: AiMapCovers;
}

/** Every step, in the order the compile runs them and the form lists them. */
export const COMPILE_STEPS: ReadonlyArray<LevelCompileStep> = [
  ELevelCompileStep.GEOMETRY,
  ELevelCompileStep.DETAILS,
  ELevelCompileStep.AI,
];

/** The steps the SDK's own compilers run; the details and the AI map are compiled by the tools themselves. */
export const SDK_COMPILE_STEPS: ReadonlySet<LevelCompileStep> = new Set([ELevelCompileStep.GEOMETRY]);

/**
 * @param steps - The steps chosen.
 * @returns Whether any of them runs one of the SDK's compilers, and so needs the SDK named.
 */
export function isSdkNeeded(steps: ReadonlySet<LevelCompileStep>): boolean {
  return [...steps].some((step: LevelCompileStep) => SDK_COMPILE_STEPS.has(step));
}

/**
 * Whether the compile the form names cannot run: a step is chosen but the game data or the textures root is not
 * named, or a step the SDK runs is chosen without the SDK.
 *
 * @param choice - What the form names.
 * @returns Why it cannot run, or null where it can or no step is chosen.
 */
export function getCompileProblem(choice: ICompileChoice): Nullable<string> {
  if (choice.steps.size === 0) {
    return null;
  }

  if (!choice.sdk && isSdkNeeded(choice.steps)) {
    return "Name the SDK's editors directory to compile the geometry";
  }

  if (!choice.gamedata) {
    return "Name the game data the compilers read their libraries from";
  }

  if (!choice.textures) {
    return "Name the textures root the compilers read the textures from";
  }

  return null;
}

/**
 * The compile request for the steps the form chose, run on the directory the build writes; none where no step is
 * chosen, so the build runs alone.
 *
 * @param choice - What the form names; {@link getCompileProblem} must have found nothing.
 * @returns The request, or null.
 */
export function toCompileRequest(choice: ICompileChoice): Nullable<LevelCompileRequest> {
  if (choice.steps.size === 0 || getCompileProblem(choice) || !choice.gamedata || !choice.textures) {
    return null;
  }

  return {
    // The backend runs the compile on the build's destination, forces it as the build is forced, and gives it the DXC
    // the app bundles, whatever is named here.
    level: "",
    steps: COMPILE_STEPS.filter((step: LevelCompileStep) => choice.steps.has(step)),
    sdk: isSdkNeeded(choice.steps) ? choice.sdk : null,
    gamedata: choice.gamedata,
    textures: [choice.textures],
    work: null,
    isForced: false,
    backend: choice.rayBackend,
    dxc: null,
    gpu: choice.rayGpu,
    aiCovers: choice.aiCovers,
  };
}
