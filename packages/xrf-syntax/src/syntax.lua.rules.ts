import { ESyntaxToken, ISyntaxRule } from "#/syntax.types";

const LUA_KEYWORDS: string =
  "and|break|do|elseif|else|end|false|for|class|function|goto|if|in|local|nil|not|or|" +
  "repeat|return|then|true|until|while";

const LUA_BUILTINS: string =
  "self|_G|assert|error|ipairs|pairs|pcall|print|require|select|setmetatable|time_global|" +
  "getmetatable|tonumber|tostring|type|unpack|string|table|math|io|os|coroutine";

/**
 * Lua, covering both `.script` game scripts and the `.s` shader scripts, which are Lua as well.
 */
export const LUA_RULES: Array<ISyntaxRule> = [
  // Long forms first: `--[[` also matches the line comment pattern, and would end at the first newline.
  { token: ESyntaxToken.COMMENT, pattern: "--\\[\\[[\\s\\S]*?\\]\\]|--\\[=\\[[\\s\\S]*?\\]=\\]|--[^\\n]*" },
  {
    token: ESyntaxToken.STRING,
    pattern: "\\[\\[[\\s\\S]*?\\]\\]|\\[=\\[[\\s\\S]*?\\]=\\]|\"(?:\\\\.|[^\"\\\\\\n])*\"|'(?:\\\\.|[^'\\\\\\n])*'",
  },
  { token: ESyntaxToken.NUMBER, pattern: "\\b0[xX][0-9a-fA-F]+\\b|\\b\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b" },
  { token: ESyntaxToken.KEYWORD, pattern: `\\b(?:${LUA_KEYWORDS})\\b` },
  { token: ESyntaxToken.TYPE, pattern: `\\b(?:${LUA_BUILTINS})\\b` },
  { token: ESyntaxToken.OPERATOR, pattern: "\\.\\.\\.|\\.\\.|[=~<>]=|[+\\-*/%^#=<>(){}\\[\\];:,.]" },
];
