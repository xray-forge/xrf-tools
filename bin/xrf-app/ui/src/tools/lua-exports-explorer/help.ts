import { IToolHelp } from "@/core/routing/tool";

export const LUA_EXPORTS_EXPLORER_HELP: IToolHelp = {
  summary:
    "Planned scope: inspect Lua script namespaces, global declarations and exported functions, including signatures, comments and source locations, with search and exportable manifests.",
  limitations: [
    "This tool is a roadmap placeholder. Its planned workflow is not implemented yet.",
    "Dynamic declarations may not be recoverable through static inspection. Scripts will not be executed to discover exports.",
  ],
};
