import { IApplicationHelp } from "@/core/routing/application";

export const LUA_EXPORTS_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Planned scope: inspect Lua script namespaces, global declarations and exported functions, including signatures, comments and source locations, with search and exportable manifests.",
  limitations: [
    "This application is a roadmap placeholder. Its planned workflow is not implemented yet.",
    "Dynamic declarations may not be recoverable through static inspection. Scripts will not be executed to discover exports.",
  ],
};
