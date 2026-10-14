import { ESyntaxToken, ISyntaxRule } from "#/syntax.types";

const TYPESCRIPT_KEYWORDS: string =
  "abstract|as|async|await|break|case|catch|class|const|continue|debugger|declare|default|delete|do|else|" +
  "enum|export|extends|finally|for|from|function|get|if|implements|import|in|instanceof|interface|is|keyof|" +
  "let|namespace|new|of|private|protected|public|readonly|return|satisfies|set|static|super|switch|this|" +
  "throw|try|type|typeof|var|while|yield";

const TYPESCRIPT_TYPES: string =
  "string|number|boolean|bigint|symbol|object|unknown|never|void|any|null|undefined|true|false|" +
  "Array|Record|Promise|Partial|Readonly|Map|Set";

/**
 * TypeScript, which is what the engine's extern declarations are written in.
 */
export const TYPESCRIPT_RULES: Array<ISyntaxRule> = [
  { token: ESyntaxToken.COMMENT, pattern: "/\\*[\\s\\S]*?\\*/|//[^\\n]*" },
  {
    token: ESyntaxToken.STRING,
    pattern: "`(?:\\\\.|[^`\\\\])*`|\"(?:\\\\.|[^\"\\\\\\n])*\"|'(?:\\\\.|[^'\\\\\\n])*'",
  },
  { token: ESyntaxToken.NUMBER, pattern: "\\b0[xX][0-9a-fA-F]+\\b|\\b\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b" },
  { token: ESyntaxToken.KEYWORD, pattern: `\\b(?:${TYPESCRIPT_KEYWORDS})\\b` },
  { token: ESyntaxToken.TYPE, pattern: `\\b(?:${TYPESCRIPT_TYPES})\\b` },
  // The call every declaration is built from, worth spotting at a glance in a body.
  { token: ESyntaxToken.DIRECTIVE, pattern: "\\bextern\\b" },
  { token: ESyntaxToken.KEY, pattern: "[A-Za-z_$][\\w$]*(?=\\s*\\()" },
  { token: ESyntaxToken.OPERATOR, pattern: "=>|\\.\\.\\.|[+\\-*/%=<>!&|^~?:;,.(){}\\[\\]]" },
];
