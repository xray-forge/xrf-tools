import { IToolHelp } from "@/core/routing/tool";

export const LUA_VERIFIER_HELP: IToolHelp = {
  summary:
    "Planned scope: verify Lua scripts against the selected X-Ray Lua dialect, report syntax and statically detectable compatibility or reference problems with file and line locations, and support project-wide checks.",
  limitations: [
    "This tool is a roadmap placeholder. Its planned workflow is not implemented yet.",
    "Static checks cannot prove runtime behavior or resolve every dynamically constructed reference.",
  ],
};
