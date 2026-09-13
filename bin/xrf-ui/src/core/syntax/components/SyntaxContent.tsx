import { Box } from "@mui/material";
import { Fragment, ReactElement, useMemo } from "react";

import { getSyntaxSx } from "@/core/syntax/components/syntax.styles";
import { ESyntaxLanguage, ESyntaxToken, highlightSyntax, ISyntaxSpan } from "@/core/syntax/lib";

interface ISyntaxContentProps {
  content: string;
  language: ESyntaxLanguage;
}

/**
 * Source text, coloured by its grammar.
 */
export function SyntaxContent({ content, language }: ISyntaxContentProps): ReactElement {
  const spans: Array<ISyntaxSpan> = useMemo(() => highlightSyntax(content, language), [content, language]);

  return (
    <Box component={"span"} sx={getSyntaxSx}>
      {spans.map((span: ISyntaxSpan, index: number) =>
        span.token === ESyntaxToken.PLAIN ? (
          <Fragment key={index}>{span.text}</Fragment>
        ) : (
          <span key={index} data-syntax-token={span.token}>
            {span.text}
          </span>
        )
      )}
    </Box>
  );
}
