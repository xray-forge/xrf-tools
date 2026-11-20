import {
  AiMapCovers,
  EAiMapCovers,
  ELevelCompileStep,
  ELightQuality,
  ERayBackend,
  LevelCompileStep,
  LightQuality,
  RayBackend,
} from "@xrf/ipc/types/xrf-level-compiler";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

/** How the form names each compile step, after the SDK compiler that step stands in for. */
export const COMPILE_STEP_LABELS: Record<LevelCompileStep, { label: string; description: string }> = {
  [ELevelCompileStep.GEOMETRY]: {
    label: "Compile geometry",
    description: "level, level.geom, level.geomx and level.cform from build.prj, as xrLC builds them",
  },
  [ELevelCompileStep.LIGHT]: {
    label: "Light geometry",
    description:
      "The light of the vertices, lightmaps and terrain, baked in the same pass as the geometry; without it the " +
      "level gets flat neutral light",
  },
  [ELevelCompileStep.DETAILS]: {
    label: "Light details",
    description: "level.details lit from build.details, build.cform and build.lights, as xrDO_Light lights it",
  },
  [ELevelCompileStep.AI]: {
    label: "Compile AI map",
    description:
      "level.ai from build.aimap, as xrAI -f compiles it, its covers traced against build.cform, then checked as a graph",
  },
};

export const RAY_BACKEND_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RayBackend>> = [
  { label: "Auto", value: ERayBackend.AUTO },
  { label: "GPU", value: ERayBackend.GPU },
  { label: "CPU", value: ERayBackend.CPU },
];

export const AI_COVERS_OPTIONS: ReadonlyArray<IChoiceFormRowOption<AiMapCovers>> = [
  { label: "Full", value: EAiMapCovers.FULL },
  { label: "None", value: EAiMapCovers.NONE },
];

export const LIGHT_QUALITY_OPTIONS: ReadonlyArray<IChoiceFormRowOption<LightQuality>> = [
  { label: "Scene", value: ELightQuality.SCENE },
  { label: "Draft", value: ELightQuality.DRAFT },
];
