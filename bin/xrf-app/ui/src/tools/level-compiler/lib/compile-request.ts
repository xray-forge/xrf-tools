import {
  AiMapCovers,
  ELevelCompileStep,
  LevelCompileRequest,
  LevelCompileStep,
  LightQuality,
  RayBackend,
  RayGpu,
} from "@xrf/ipc/types/xrf-level-compiler";
import { Nullable } from "@xrf/types";

import { ELevelCompilerField } from "@/core/level/lib/compiler/level-compiler-field";

/** What the form names for the compile after the build. */
export interface ICompileChoice {
  steps: ReadonlySet<LevelCompileStep>;
  /** The game materials the build writes `build.prj` from, which every step reads. */
  gameMaterials: Nullable<string>;
  /** The game data holding `gamemtl.xr` and `shaders_xrlc.xr`. */
  gamedata: Nullable<string>;
  /**
   * Textures root read before the game data's own, such as a decompile's impostor strips; null for the game data's
   * alone. Together they must hold a `.thm` for every texture `build.prj` names.
   */
  textures: Nullable<string>;
  /** What traces the light, details and AI cover rays. */
  rayBackend: RayBackend;
  /** GPU the rays trace on; null for the automatic one. */
  rayGpu: Nullable<RayGpu>;
  /** AI map covers: traced against `build.cform`, or none. */
  aiCovers: AiMapCovers;
  /** Light step quality: `build.prj`'s own sky and sun, or a cheap draft. */
  lightQuality: LightQuality;
}

/** Every step the form offers, in run order. */
export const COMPILE_STEPS: ReadonlyArray<LevelCompileStep> = [
  ELevelCompileStep.GEOMETRY,
  ELevelCompileStep.LIGHT,
  ELevelCompileStep.DETAILS,
  ELevelCompileStep.AI,
];

/**
 * The steps chosen after one step's box changes.
 *
 * The light is baked in the geometry's pass: ticking the geometry ticks the light, unticking it drops the light, and
 * the result never holds the light without the geometry.
 */
export function toggleCompileStep(
  steps: ReadonlySet<LevelCompileStep>,
  step: LevelCompileStep,
  isChecked: boolean
): ReadonlySet<LevelCompileStep> {
  const next: Set<LevelCompileStep> = new Set(steps);

  if (isChecked) {
    next.add(step);

    if (step === ELevelCompileStep.GEOMETRY) {
      next.add(ELevelCompileStep.LIGHT);
    }
  } else {
    next.delete(step);

    if (step === ELevelCompileStep.GEOMETRY) {
      next.delete(ELevelCompileStep.LIGHT);
    }
  }

  if (!next.has(ELevelCompileStep.GEOMETRY)) {
    next.delete(ELevelCompileStep.LIGHT);
  }

  return next;
}

/**
 * The steps chosen that the scene can be compiled with: the details step only where it may hold details, so a box
 * ticked for another scene is not sent for this one.
 *
 * @param steps - The steps ticked.
 * @param hasDetails - Whether the scene holds details; null while unknown, which keeps the step.
 */
export function getOfferedSteps(
  steps: ReadonlySet<LevelCompileStep>,
  hasDetails: Nullable<boolean>
): ReadonlySet<LevelCompileStep> {
  if (hasDetails !== false || !steps.has(ELevelCompileStep.DETAILS)) {
    return steps;
  }

  const offered: Set<LevelCompileStep> = new Set(steps);

  offered.delete(ELevelCompileStep.DETAILS);

  return offered;
}

/** Why the compile the form names cannot run, and the field that answers it. */
export interface ICompileProblem {
  /** The path field to fill; null for the steps chosen. */
  field: Nullable<ELevelCompilerField>;
  message: string;
}

/**
 * Why the compile the form names cannot run: light without geometry, or no game materials or game data named.
 *
 * @returns The problem, or null where it can run or no step is chosen.
 */
export function getCompileProblem(choice: ICompileChoice): Nullable<ICompileProblem> {
  if (choice.steps.size === 0) {
    return null;
  }

  if (choice.steps.has(ELevelCompileStep.LIGHT) && !choice.steps.has(ELevelCompileStep.GEOMETRY)) {
    return { field: null, message: "The light is baked with the geometry; tick the geometry to light the level" };
  }

  if (!choice.gameMaterials) {
    return {
      field: ELevelCompilerField.GAME_MATERIALS,
      message: "Name the game materials: the compile reads the build.prj the build writes from them",
    };
  }

  if (!choice.gamedata) {
    return {
      field: ELevelCompilerField.GAMEDATA,
      message: "Name the game data the compilers read their libraries and textures from",
    };
  }

  return null;
}

/**
 * The compile request for the chosen steps, run on the directory the build writes.
 *
 * @returns The request, or null where no step is chosen or {@link getCompileProblem} finds one, so the build runs alone.
 */
export function toCompileRequest(choice: ICompileChoice): Nullable<LevelCompileRequest> {
  if (choice.steps.size === 0 || getCompileProblem(choice) || !choice.gamedata) {
    return null;
  }

  return {
    // The backend takes the level and forcing from the build, gives the compile the DXC the app bundles, and reads the
    // game data's textures after these roots.
    level: "",
    steps: COMPILE_STEPS.filter((step: LevelCompileStep) => choice.steps.has(step)),
    gamedata: choice.gamedata,
    textures: choice.textures ? [choice.textures] : [],
    isForced: false,
    backend: choice.rayBackend,
    dxc: null,
    gpu: choice.rayGpu,
    aiCovers: choice.aiCovers,
    lightQuality: choice.lightQuality,
  };
}
