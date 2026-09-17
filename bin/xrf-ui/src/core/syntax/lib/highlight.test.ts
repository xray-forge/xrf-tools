import { describe, expect, it } from "@jest/globals";

import { highlightSyntax } from "@/core/syntax/lib/highlight";
import { getSyntaxRules, isLineLocalSyntax } from "@/core/syntax/lib/syntax-rules";
import { ESyntaxLanguage, ESyntaxToken, ISyntaxRule, ISyntaxSpan } from "@/core/syntax/lib/syntax.types";

const LANGUAGES: Array<ESyntaxLanguage> = [
  ESyntaxLanguage.LTX,
  ESyntaxLanguage.LUA,
  ESyntaxLanguage.SHADER,
  ESyntaxLanguage.TYPESCRIPT,
  ESyntaxLanguage.XML,
];

/**
 * Text of the first span carrying a token, which is how these tests ask "was this coloured".
 *
 * @param spans - Result of a highlight pass.
 * @param token - Token to look for.
 * @returns Every matching span's text, in order.
 */
function textOf(spans: Array<ISyntaxSpan>, token: ESyntaxToken): Array<string> {
  return spans.filter((span: ISyntaxSpan) => span.token === token).map((span: ISyntaxSpan) => span.text);
}

describe("highlightSyntax", () => {
  it.each(LANGUAGES)("reproduces the input exactly for %s", (language: ESyntaxLanguage) => {
    // The single property everything else rests on: colouring may not add, drop or reorder a character.
    const content: string =
      '; comment\n[section]:parent\nkey = "value", 12.5\n<tag attr="x"/>\n--[[ lua ]]\n/* c */ #include "a.h"\n';

    expect(
      highlightSyntax(content, language)
        .map((span: ISyntaxSpan) => span.text)
        .join("")
    ).toBe(content);
  });

  it("leaves an unknown language untouched rather than guessing", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax("anything at all", ESyntaxLanguage.PLAIN);

    expect(spans).toEqual([{ token: ESyntaxToken.PLAIN, text: "anything at all" }]);
  });

  it("handles empty content", () => {
    expect(highlightSyntax("", ESyntaxLanguage.LUA)).toEqual([]);
  });
});

describe("highlightSyntax over ltx", () => {
  it("colours a section apart from the parents it inherits", () => {
    // The inheritance suffix is what a stock ini grammar has no concept of, and it is the first thing
    // worth seeing when reading a weapon or npc config.
    const spans: Array<ISyntaxSpan> = highlightSyntax("[wpn_ak74]:wpn_base, wpn_shared\n", ESyntaxLanguage.LTX);

    expect(textOf(spans, ESyntaxToken.SECTION)).toEqual(["[wpn_ak74]"]);
    expect(textOf(spans, ESyntaxToken.TYPE)).toEqual(["wpn_base, wpn_shared"]);
  });

  it("separates a key from its value", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax("ammo_mag_size = 30\n", ESyntaxLanguage.LTX);

    expect(textOf(spans, ESyntaxToken.KEY)).toEqual(["ammo_mag_size"]);
    expect(textOf(spans, ESyntaxToken.NUMBER)).toEqual(["30"]);
  });

  it("colours includes, since a config is rarely one file", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax('#include "weapons\\wpn_ak74.ltx"\n', ESyntaxLanguage.LTX);

    expect(textOf(spans, ESyntaxToken.DIRECTIVE)).toEqual(["#include"]);
    expect(textOf(spans, ESyntaxToken.STRING)).toEqual(['"weapons\\wpn_ak74.ltx"']);
  });

  it("does not read a key out of a comment", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax("; disabled = 1\n", ESyntaxLanguage.LTX);

    expect(textOf(spans, ESyntaxToken.COMMENT)).toEqual(["; disabled = 1"]);
    expect(textOf(spans, ESyntaxToken.KEY)).toEqual([]);
  });
});

describe("highlightSyntax over lua", () => {
  it("keeps a long comment whole rather than ending it at the first newline", () => {
    // `--[[` also matches the line comment rule, so this only works while the long form is tried first.
    const spans: Array<ISyntaxSpan> = highlightSyntax("--[[ first\nsecond ]]\nlocal a = 1\n", ESyntaxLanguage.LUA);

    expect(textOf(spans, ESyntaxToken.COMMENT)).toEqual(["--[[ first\nsecond ]]"]);
    expect(textOf(spans, ESyntaxToken.KEYWORD)).toEqual(["local"]);
  });

  it("does not colour keywords found inside a string", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax('local name = "end of function"\n', ESyntaxLanguage.LUA);

    expect(textOf(spans, ESyntaxToken.STRING)).toEqual(['"end of function"']);
    expect(textOf(spans, ESyntaxToken.KEYWORD)).toEqual(["local"]);
  });

  it("does not colour a keyword that is only part of an identifier", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax("local ending = fortune\n", ESyntaxLanguage.LUA);

    expect(textOf(spans, ESyntaxToken.KEYWORD)).toEqual(["local"]);
  });

  it("reads the shader scripts that are lua despite their extension", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax(
      'function normal(shader)\n\tshader:begin("null","avg2")\nend\n',
      ESyntaxLanguage.LUA
    );

    expect(textOf(spans, ESyntaxToken.KEYWORD)).toEqual(["function", "end"]);
    expect(textOf(spans, ESyntaxToken.STRING)).toEqual(['"null"', '"avg2"']);
  });
});

describe("highlightSyntax over shaders", () => {
  it("colours a preprocessor line that is spaced out", () => {
    // These files are full of `# ifndef`, which a pattern demanding `#ifndef` would miss.
    const spans: Array<ISyntaxSpan> = highlightSyntax(
      '# ifndef USE_SHADOW\n#include "common.h"\n',
      ESyntaxLanguage.SHADER
    );

    expect(textOf(spans, ESyntaxToken.DIRECTIVE)).toEqual(["# ifndef", "#include"]);
  });

  it("colours vector types by their width", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax(
      "uniform half4 L_dynamic;\nfloat4x4 m;\n",
      ESyntaxLanguage.SHADER
    );

    expect(textOf(spans, ESyntaxToken.TYPE)).toEqual(["half4", "float4x4"]);
    expect(textOf(spans, ESyntaxToken.KEYWORD)).toEqual(["uniform"]);
  });

  it("keeps a block comment whole across lines", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax("/* one\ntwo */ float a;\n", ESyntaxLanguage.SHADER);

    expect(textOf(spans, ESyntaxToken.COMMENT)).toEqual(["/* one\ntwo */"]);
  });
});

describe("highlightSyntax over xml", () => {
  it("separates tag names, attributes and their values", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax('<string id="ui_menu">text</string>', ESyntaxLanguage.XML);

    expect(textOf(spans, ESyntaxToken.SECTION)).toEqual(["<string", "</string"]);
    expect(textOf(spans, ESyntaxToken.KEY)).toEqual(["id"]);
    expect(textOf(spans, ESyntaxToken.STRING)).toEqual(['"ui_menu"']);
  });

  it("does not read markup out of a comment", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax("<!-- <string id='a'> -->", ESyntaxLanguage.XML);

    expect(textOf(spans, ESyntaxToken.COMMENT)).toEqual(["<!-- <string id='a'> -->"]);
    expect(textOf(spans, ESyntaxToken.SECTION)).toEqual([]);
  });
});

describe("syntax rules", () => {
  it.each(LANGUAGES)("declares no capturing groups in %s", (language: ESyntaxLanguage) => {
    // The combined scanner identifies a rule by capture group index, so an inner group in any pattern
    // would silently shift every rule after it and mislabel their tokens.
    for (const rule of getSyntaxRules(language)) {
      const groups: number = (new RegExp(`${rule.pattern}|`).exec("") as RegExpExecArray).length - 1;

      expect({ pattern: rule.pattern, groups }).toEqual({ pattern: rule.pattern, groups: 0 });
    }
  });

  it.each(LANGUAGES)("agrees with what its rules do to a newline in %s", (language: ESyntaxLanguage) => {
    // A listing colours a line at a time only where this holds, so a rule added without moving the language
    // out of the line-local set would silently mis-colour every window drawn after the one it opens.
    // Every construct that reaches past a newline in any of these grammars, opened and closed a line apart.
    const probe: string = [
      "; comment",
      "[section]:parent",
      'key = "unterminated',
      "--[[ long",
      "still here ]]",
      "/* block",
      "still here */",
      "<!-- markup",
      "still here -->",
      "`template",
      "still here`",
    ].join("\n");
    const crossing: Array<ISyntaxSpan> = highlightSyntax(probe, language).filter(
      (span: ISyntaxSpan) => span.token !== ESyntaxToken.PLAIN && span.text.includes("\n")
    );

    expect({ crosses: crossing.length > 0, language }).toEqual({ crosses: !isLineLocalSyntax(language), language });
  });

  it.each(LANGUAGES)("declares no rule that can match nothing in %s", (language: ESyntaxLanguage) => {
    // A zero-length match would stall the scan, and the loop only survives it by stepping over it.
    for (const rule of getSyntaxRules(language) as Array<ISyntaxRule>) {
      expect({ pattern: rule.pattern, empty: new RegExp(`^(?:${rule.pattern})$`).test("") }).toEqual({
        pattern: rule.pattern,
        empty: false,
      });
    }
  });
});

describe("highlightSyntax over typescript", () => {
  it("colours an extern declaration the way the exports editor shows it", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax(
      'extern("xr_effects.run", (actor: number): void => {});',
      ESyntaxLanguage.TYPESCRIPT
    );

    expect(textOf(spans, ESyntaxToken.DIRECTIVE)).toEqual(["extern"]);
    expect(textOf(spans, ESyntaxToken.STRING)).toEqual(['"xr_effects.run"']);
    expect(textOf(spans, ESyntaxToken.TYPE)).toEqual(["number", "void"]);
  });

  it("keeps a template literal whole across lines", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax("const a = `one\ntwo`;\n", ESyntaxLanguage.TYPESCRIPT);

    expect(textOf(spans, ESyntaxToken.STRING)).toEqual(["`one\ntwo`"]);
  });

  it("does not colour a keyword that is only part of an identifier", () => {
    const spans: Array<ISyntaxSpan> = highlightSyntax("const constant = 1;", ESyntaxLanguage.TYPESCRIPT);

    expect(textOf(spans, ESyntaxToken.KEYWORD)).toEqual(["const"]);
  });
});
