import { CommandType, QueryType } from "@wirestate/core";

/** Leaves the open tool for the home screen at once; whoever sends it has already asked about unsaved work. */
export const CLOSE_TOOL_COMMAND: CommandType = Symbol("@/shell/close-tool");

/** Answers whether a tool is open, rather than the home screen. */
export const TOOL_OPEN_QUERY: QueryType = Symbol("@/shell/tool-open");
