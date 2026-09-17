import { Fragment, ReactElement, useMemo } from "react";

import { ESyntaxLanguage, ESyntaxToken, highlightSyntax, ISyntaxSpan } from "@/core/syntax/lib";

/** Size past which text is shown uncoloured. */
export const MAXIMUM_HIGHLIGHT_LENGTH: number = 512 * 1024;

interface ISyntaxContentProps {
  content: string;
  language: ESyntaxLanguage;
}

/**
 * Source text, coloured by its grammar.
 */
export function SyntaxContent({ content, language }: ISyntaxContentProps): Array<ReactElement> {
  const spans: Array<ISyntaxSpan> = useMemo(
    () =>
      content.length > MAXIMUM_HIGHLIGHT_LENGTH
        ? [{ token: ESyntaxToken.PLAIN, text: content }]
        : highlightSyntax(content, language),
    [content, language]
  );

  return spans.map((span: ISyntaxSpan, index: number) =>
    span.token === ESyntaxToken.PLAIN ? (
      <Fragment key={index}>{span.text}</Fragment>
    ) : (
      <span key={index} data-syntax-token={span.token}>
        {span.text}
      </span>
    )
  );
}
