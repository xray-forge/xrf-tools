import { ESyntaxToken, ISyntaxRule } from "#/syntax.types";

const SHADER_KEYWORDS: string =
  "uniform|static|const|extern|shared|volatile|inline|typedef|namespace|return|if|else|for|while|do|" +
  "switch|case|default|break|continue|discard|in|out|inout|register|packoffset|cbuffer|tbuffer|" +
  "technique|pass|compile|struct|true|false";

const SHADER_TYPES: string =
  "(?:float|half|double|int|uint|min16float|min10float|bool)[1-4]?(?:x[1-4])?|" +
  "void|matrix|vector|string|sampler(?:1D|2D|3D|CUBE|_state)?|SamplerState|SamplerComparisonState|" +
  "Texture(?:1D|2D|3D|Cube)(?:Array)?|(?:RW)?(?:Structured|ByteAddress)?Buffer|InputPatch|OutputPatch|" +
  "PointStream|LineStream|TriangleStream";

/**
 * HLSL, as used by `.ps`, `.vs`, `.hs`, `.ds` and the `.h` headers they include.
 */
export const SHADER_RULES: Array<ISyntaxRule> = [
  { token: ESyntaxToken.COMMENT, pattern: "/\\*[\\s\\S]*?\\*/|//[^\\n]*" },
  // The space is real: these files are full of `# ifndef` and `#  define`.
  { token: ESyntaxToken.DIRECTIVE, pattern: "^[ \\t]*#[ \\t]*[A-Za-z_]+" },
  { token: ESyntaxToken.STRING, pattern: '"[^"\\n]*"' },
  {
    token: ESyntaxToken.NUMBER,
    pattern: "\\b0[xX][0-9a-fA-F]+\\b|\\b\\d+\\.?\\d*(?:[eE][+-]?\\d+)?[fFhHlLuU]?\\b|\\.\\d+[fFhH]?\\b",
  },
  { token: ESyntaxToken.TYPE, pattern: `\\b(?:${SHADER_TYPES})\\b` },
  { token: ESyntaxToken.KEYWORD, pattern: `\\b(?:${SHADER_KEYWORDS})\\b` },
  { token: ESyntaxToken.OPERATOR, pattern: "[+\\-*/%=<>!&|^~?:;,.(){}\\[\\]]" },
];
