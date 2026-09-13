import { Fragment, ReactElement, useMemo } from "react";

import { ESyntaxLanguage, ESyntaxToken, highlightSyntax, ISyntaxSpan } from "@/core/syntax/lib";

interface ISyntaxContentProps {
  content: string;
  language: ESyntaxLanguage;
}

/**
 * Source text, coloured by its grammar.
 */
export function SyntaxContent({ content, language }: ISyntaxContentProps): Array<ReactElement> {
  const spans: Array<ISyntaxSpan> = useMemo(() => highlightSyntax(content, language), [content, language]);

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
