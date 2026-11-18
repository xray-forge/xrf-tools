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
   * The textures root read before the game data's own `textures`, such as the one a decompile wrote impostor strips
   * into; none to read the game data's alone. Between them they hold a `.thm` for every texture `build.prj` names.
   */
  textures: Nullable<string>;
  /** What traces the rays: the light's lightmap and terrain rays, the details' shadow rays and the AI map's cover rays. */
  rayBackend: RayBackend;
  /** The GPU they trace on, the renderer's choice; null for the automatic one. */
  rayGpu: Nullable<RayGpu>;
  /** The covers the AI map gets: full ones traced against `build.cform`, or none. */
  aiCovers: AiMapCovers;
  /** How finely the light step lights the level: by `build.prj`'s own sky and sun, or a cheap draft. */
  lightQuality: LightQuality;
}

/** Every step the form offers, in the order the compile runs them and the form lists them. */
export const COMPILE_STEPS: ReadonlyArray<LevelCompileStep> = [
  ELevelCompileStep.GEOMETRY,
  ELevelCompileStep.LIGHT,
  ELevelCompileStep.DETAILS,
  ELevelCompileStep.AI,
];

/**
 * The steps chosen after one step's box changes: the light is baked in the geometry's pass, so ticking the geometry
 * ticks its light too and unticking it drops the light, and a choice never holds the light without the geometry.
 *
 * @param steps - The steps chosen before.
 * @param step - The step whose box changed.
 * @param isChecked - Whether it is now ticked.
 * @returns The steps chosen after.
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

/** Why the compile the form names cannot run, and the field that answers it. */
export interface ICompileProblem {
  /** The path field to fill; null for the steps chosen. */
  field: Nullable<ELevelCompilerField>;
  message: string;
}

/**
 * Whether the compile the form names cannot run: a step is chosen but the game materials the build writes `build.prj`
 * from or the game data is not named, or the light is chosen without the geometry it is baked with.
 *
 * @param choice - What the form names.
 * @returns Why it cannot run, or null where it can or no step is chosen.
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
 * The compile request for the steps the form chose, run on the directory the build writes; none where no step is
 * chosen, so the build runs alone.
 *
 * @param choice - What the form names; {@link getCompileProblem} must have found nothing.
 * @returns The request, or null.
 */
export function toCompileRequest(choice: ICompileChoice): Nullable<LevelCompileRequest> {
  if (choice.steps.size === 0 || getCompileProblem(choice) || !choice.gamedata) {
    return null;
  }

  return {
    // The backend runs the compile on the build's destination, forces it as the build is forced, gives it the DXC the
    // app bundles, whatever is named here, and reads the game data's textures after the roots named.
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
