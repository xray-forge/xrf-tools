import { IToolHelp } from "@/core/routing/tool";

export const LUA_FORMATTER_HELP: IToolHelp = {
  summary:
    "Planned scope: format individual Lua scripts or project trees, preview changes and check formatting without writing, while preserving comments, file encoding and compatibility with the selected Lua dialect.",
  limitations: [
    "This application is a roadmap placeholder. Its planned workflow is not implemented yet.",
    "Formatting will preserve program behavior; it will not refactor or execute scripts.",
  ],
};
