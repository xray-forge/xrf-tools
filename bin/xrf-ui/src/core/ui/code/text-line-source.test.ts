import { describe, expect, it } from "@jest/globals";

import { ESyntaxLanguage, ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { ICodeLineSource } from "@/core/ui/code/code-line";
import { MAXIMUM_SCANNED_LENGTH, toTextLineSource } from "@/core/ui/code/text-line-source";

function getSpansOf(source: ICodeLineSource, index: number): Array<string> {
  return source.getLine(index).spans.map((span: ISyntaxSpan) => `${span.token}:${span.text}`);
}

function getTextOf(source: ICodeLineSource): string {
  return Array.from({ length: source.count }, (_, index: number) =>
    source
      .getLine(index)
      .spans.map((span: ISyntaxSpan) => span.text)
      .join("")
  ).join("\n");
}

describe("toTextLineSource", () => {
  it("lays a document out by its newlines, numbering from one", () => {
    const source: ICodeLineSource = toTextLineSource("[a]\nkey = 1\n", ESyntaxLanguage.LTX);

    // Three, because the trailing newline opens a line the file has not written to yet - which is the line a
    // gutter counts and an editor puts the caret on.
    expect(source.count).toBe(3);
    expect(source.getLine(0).number).toBe(1);
    expect(source.widestNumber).toBe(3);
    expect(source.indexOfLine(3)).toBe(2);
    expect(source.indexOfLine(4)).toBe(-1);
  });

  it("keeps an excerpt numbered as the file it came from", () => {
    const source: ICodeLineSource = toTextLineSource("local a = 1", ESyntaxLanguage.LUA, 40);

    expect(source.getLine(0).number).toBe(40);
    expect(source.widestNumber).toBe(40);
    expect(source.indexOfLine(40)).toBe(0);
    expect(source.indexOfLine(1)).toBe(-1);
  });

  it("reproduces what it was given, whatever it colours", () => {
    const content: string = '; comment\n[wpn_ak74]:wpn_base\nammo_mag_size = 30\n\n#include "a.ltx"';

    expect(getTextOf(toTextLineSource(content, ESyntaxLanguage.LTX))).toBe(content);
  });

  it("leaves a carriage return where the file put it", () => {
    // What the listing shows has to be what extracting the file writes, and a `\r` draws as nothing anyway.
    const source: ICodeLineSource = toTextLineSource("; comment\r\n[a]\r\n", ESyntaxLanguage.LTX);

    expect(getSpansOf(source, 0)).toEqual([`${ESyntaxToken.COMMENT}:; comment\r`]);
  });

  it("colours a line-local grammar a line at a time, however long the document is", () => {
    // The case this exists for: Anomaly's `textures.ltx` is some 700 KB, past anything one pass is worth.
    const content: string = new Array(80_000).fill("act\\act_lenin = detail\\detail_tile_det3, 6.000000").join("\n");
    const source: ICodeLineSource = toTextLineSource(content, ESyntaxLanguage.LTX);

    expect(content.length).toBeGreaterThan(MAXIMUM_SCANNED_LENGTH / 4);
    expect(source.count).toBe(80_000);
    expect(getSpansOf(source, 79_999)).toContain(`${ESyntaxToken.KEY}:act\\act_lenin`);
  });

  it("colours a line-local grammar the same one line at a time as it would whole", () => {
    // The property the lazy path rests on. An unterminated quote and a stray bracket are where a grammar that
    // only looks line-local stops being it.
    const lines: Array<string> = ['key = "unterminated', "[section]:parent ; and a comment", "#include [not a header]"];
    const source: ICodeLineSource = toTextLineSource(lines.join("\n"), ESyntaxLanguage.LTX);

    for (const [index, line] of lines.entries()) {
      expect(getSpansOf(source, index)).toEqual(
        toTextLineSource(line, ESyntaxLanguage.LTX)
          .getLine(0)
          .spans.map((span: ISyntaxSpan) => `${span.token}:${span.text}`)
      );
    }
  });

  it("reads a grammar that crosses lines over the whole document", () => {
    // A long comment read a line at a time would end at the first newline and colour its body as code.
    const source: ICodeLineSource = toTextLineSource("--[[ first\nsecond ]]\nlocal a = 1", ESyntaxLanguage.LUA);

    expect(getSpansOf(source, 0)).toEqual([`${ESyntaxToken.COMMENT}:--[[ first`]);
    expect(getSpansOf(source, 1)).toEqual([`${ESyntaxToken.COMMENT}:second ]]`]);
    expect(getSpansOf(source, 2)).toContain(`${ESyntaxToken.KEYWORD}:local`);
  });

  it("draws a document past the scan limit plainly rather than wrongly", () => {
    // Only a grammar that has to be read whole can reach this, and there the spans of that pass are held for as
    // long as the document is open.
    const line: string = '<string id="ui_menu">text</string>';
    const content: string = new Array(Math.ceil(MAXIMUM_SCANNED_LENGTH / line.length) + 1).fill(line).join("\n");
    const source: ICodeLineSource = toTextLineSource(content, ESyntaxLanguage.XML);

    expect(getSpansOf(source, 0)).toEqual([`${ESyntaxToken.PLAIN}:${line}`]);
  });

  it("holds no spans for a blank line", () => {
    const source: ICodeLineSource = toTextLineSource("[a]\n\n[b]", ESyntaxLanguage.LTX);

    expect(source.getLine(1).spans).toEqual([]);
  });

  it("answers for an empty document with the one line it shows", () => {
    const source: ICodeLineSource = toTextLineSource("", ESyntaxLanguage.LTX);

    expect(source.count).toBe(1);
    expect(source.widestNumber).toBe(1);
    expect(source.getLine(0)).toEqual({ number: 1, spans: [] });
  });

  it("identifies the document it is a view of", () => {
    const source: ICodeLineSource = toTextLineSource("[a]", ESyntaxLanguage.LTX);

    expect(source.layout).toBe(source.layout);
    expect(toTextLineSource("[a]", ESyntaxLanguage.LTX).layout).not.toBe(source.layout);
  });
});
