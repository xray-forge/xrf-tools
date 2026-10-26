export * from "./command-descriptor";
export * from "./command.decorator";
export * from "./command.plugin";

/**
 * `command-metadata.ts` is reached as a sibling by the decorator and the plugin and stays out of this barrel: it is
 * how a command binds, not something a caller declares against.
 */
