import { useTheme } from "@mui/material";
import { Fragment, ReactElement, useMemo } from "react";

import { getSyntaxColors } from "@/core/syntax/components/syntax.styles";
import { ESyntaxLanguage, ESyntaxToken, highlightSyntax, ISyntaxSpan } from "@/core/syntax/lib";

interface ISyntaxContentProps {
  content: string;
  language: ESyntaxLanguage;
}

/**
 * Source text, coloured by its grammar.
 */
export function SyntaxContent({ content, language }: ISyntaxContentProps): ReactElement {
  const theme = useTheme();

  const spans: Array<ISyntaxSpan> = useMemo(() => highlightSyntax(content, language), [content, language]);
  const colors: Record<ESyntaxToken, string> = useMemo(() => getSyntaxColors(theme), [theme]);

  return (
    <>
      {spans.map((span: ISyntaxSpan, index: number) =>
        span.token === ESyntaxToken.PLAIN ? (
          <Fragment key={index}>{span.text}</Fragment>
        ) : (
          <span key={index} style={{ color: colors[span.token] }}>
            {span.text}
          </span>
        )
      )}
    </>
  );
}
