export * from "./lib";
export * from "./services/keybind-commands.service";

/**
 * `root-commands.ts` is deliberately absent: the domains listed there import this barrel to declare their commands, so
 * re-exporting it would make the two a load-time cycle. Its consumers reach it by path.
 */
