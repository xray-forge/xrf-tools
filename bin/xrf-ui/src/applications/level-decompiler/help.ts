import { IApplicationHelp } from "@/core/routing/application";

export const LEVEL_DECOMPILER_HELP: IApplicationHelp = {
  summary:
    "Planned scope: reconstruct editable source meshes and scene records from compiled levels, resolving materials and supported collision, visibility, details and spawn data. Compilation can discard original grouping and authoring data, so reconstruction cannot promise the original source project.",
  limitations: ["This application is a roadmap placeholder. Its planned workflow is not implemented yet."],
};
