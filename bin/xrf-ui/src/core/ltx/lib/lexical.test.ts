import { describe, expect, it } from "@jest/globals";

import { toLexicalLines } from "@/core/ltx/lib/lexical";
import { ESyntaxToken, ISyntaxSpan, MAXIMUM_HIGHLIGHT_LENGTH } from "@/core/syntax/lib";

/**
 * Text of every span carrying a token, which is how these tests ask "was this coloured".
 *
 * @param spans - Spans of one line.
 * @param token - Token to look for.
 * @returns Every matching span's text, in order.
 */
function textOf(spans: Array<ISyntaxSpan>, token: ESyntaxToken): Array<string> {
  return spans.filter((span: ISyntaxSpan) => span.token === token).map((span: ISyntaxSpan) => span.text);
}

/**
 * Spans of a single line, for a trap that is about one line and nothing around it.
 *
 * @param line - Source line, without its terminator.
 * @returns Its spans.
 */
function lex(line: string): Array<ISyntaxSpan> {
  return toLexicalLines([line])[0];
}

describe("toLexicalLines", () => {
  it("answers one entry per input line, in order", () => {
    const lines: Array<string> = ["[wpn_ak74]", "cost = 1000", "", "; end"];

    expect(toLexicalLines(lines)).toHaveLength(4);
    expect(toLexicalLines(lines).map((spans: Array<ISyntaxSpan>) => spans.map((it) => it.text).join(""))).toEqual(
      lines
    );
  });

  it("colours nothing on an empty line and leaves a whitespace-only one plain", () => {
    expect(lex("")).toEqual([]);
    // The Authored view never sees this case today: `LtxDocument::get_source_lines` normalizes a
    // whitespace-only line to empty, which `LtxFileText.is_normalized` records. A resolved or edited
    // document can still produce one, and indentation is not something to colour.
    expect(lex("   ")).toEqual([{ token: ESyntaxToken.PLAIN, text: "   " }]);
  });

  it("keeps the inner spaces of a section name, which the engine counts as part of it", () => {
    // `[  wpn_base  ]` and `[wpn_base]` are two different sections: `parse_section_from_line` takes the
    // text between the brackets verbatim, without trimming. The colouring is faithful because the span
    // carries the spaces, but it cannot say the two headers differ - only the structure record can.
    expect(lex("[  wpn_base  ]")).toEqual([{ token: ESyntaxToken.SECTION, text: "[  wpn_base  ]" }]);
    expect(lex("[wpn_base]")).toEqual([{ token: ESyntaxToken.SECTION, text: "[wpn_base]" }]);
  });

  it("colours a header apart from the parents it inherits", () => {
    const spans: Array<ISyntaxSpan> = lex("[wpn_ak74]:wpn_base,identity_immunities");

    expect(textOf(spans, ESyntaxToken.SECTION)).toEqual(["[wpn_ak74]"]);
    expect(textOf(spans, ESyntaxToken.TYPE)).toEqual(["wpn_base,identity_immunities"]);
  });

  it("colours an include and the quoted path it names", () => {
    const spans: Array<ISyntaxSpan> = lex('#include "items\\w_*.ltx"');

    expect(textOf(spans, ESyntaxToken.DIRECTIVE)).toEqual(["#include"]);
    expect(textOf(spans, ESyntaxToken.STRING)).toEqual(['"items\\w_*.ltx"']);
  });

  it("does not recognise a DLTX operation prefix on a header", () => {
    // A header is anchored at line start, so `!`, `@` and `!!` push the bracket off the anchor and the
    // whole line falls through to plain. Left as it is on purpose: the structure record describes header
    // lines outright and replaces these spans, so the regex is not the thing that has to be right here
    // (`plans/configs-explorer.md` decision 4).
    expect(lex("![wpn_ak74]")).toEqual([{ token: ESyntaxToken.PLAIN, text: "![wpn_ak74]" }]);
    expect(lex("@[s]")).toEqual([{ token: ESyntaxToken.PLAIN, text: "@[s]" }]);
    expect(lex("!![s]")).toEqual([{ token: ESyntaxToken.PLAIN, text: "!![s]" }]);
  });

  it("does not recognise a DLTX operation prefix on a key", () => {
    // The same anchor, and this one the structure record does not cover: keys and values do not travel
    // in v1. So a patched key reads as an uncoloured line with its `=` marked, which is wrong but not
    // misleading - nothing claims `>key` is a key. Fixing it means teaching the rules the prefixes,
    // which would then have to agree with `split_statement_operation` about which ones exist.
    expect(lex(">key = a")).toEqual([
      { token: ESyntaxToken.PLAIN, text: ">key " },
      { token: ESyntaxToken.OPERATOR, text: "=" },
      { token: ESyntaxToken.PLAIN, text: " a" },
    ]);
    expect(textOf(lex("<key = b"), ESyntaxToken.KEY)).toEqual([]);
    // A removal carries no `=`, and a key is only recognised by what precedes one.
    expect(lex("!key")).toEqual([{ token: ESyntaxToken.PLAIN, text: "!key" }]);
  });

  it("ends a value at the semicolon that opens a comment", () => {
    // `;` is `LTX_SYMBOL_COMMENT` and starts a comment wherever it appears, so a semicolon inside a
    // value is not a case: the parser splits there too.
    const spans: Array<ISyntaxSpan> = lex("cost = 1000 ;in rubles");

    expect(textOf(spans, ESyntaxToken.KEY)).toEqual(["cost"]);
    expect(textOf(spans, ESyntaxToken.NUMBER)).toEqual(["1000"]);
    expect(textOf(spans, ESyntaxToken.COMMENT)).toEqual([";in rubles"]);
  });

  it("keeps a semicolon inside a quoted value out of the comment", () => {
    const spans: Array<ISyntaxSpan> = lex('path = "a;b"');

    expect(textOf(spans, ESyntaxToken.STRING)).toEqual(['"a;b"']);
    expect(textOf(spans, ESyntaxToken.COMMENT)).toEqual([]);
  });

  it("colours a double slash as a comment, which the parser does not agree with", () => {
    // `xrf-ltx` knows one comment symbol, `;`, so it reads `// in rubles` as part of the value while
    // this colours it as a comment. The disagreement is recorded rather than fixed: `//` is a comment
    // in hand-written configs across the forks these tools read, and dropping the rule would leave
    // those lines looking like data. A value containing `//` - a path written with forward slashes -
    // is mis-coloured, and that is the price.
    expect(textOf(lex("cost = 1000 // in rubles"), ESyntaxToken.COMMENT)).toEqual(["// in rubles"]);
  });

  it("does not read a key out of a comment line", () => {
    const spans: Array<ISyntaxSpan> = lex("; disabled = 1");

    expect(textOf(spans, ESyntaxToken.COMMENT)).toEqual(["; disabled = 1"]);
    expect(textOf(spans, ESyntaxToken.KEY)).toEqual([]);
  });

  it("colours a scheme binding like any other key", () => {
    expect(textOf(lex("$scheme = weapon"), ESyntaxToken.KEY)).toEqual(["$scheme"]);
  });

  it("gives up colouring a document larger than the highlight budget", () => {
    // Past the budget the spans themselves are the cost, since the listing above this renders a window
    // rather than the document.
    const line: string = "cost = 1000";
    const lines: Array<string> = new Array(Math.ceil(MAXIMUM_HIGHLIGHT_LENGTH / line.length) + 1).fill(line);

    expect(toLexicalLines(lines)[0]).toEqual([{ token: ESyntaxToken.PLAIN, text: line }]);
  });
});
