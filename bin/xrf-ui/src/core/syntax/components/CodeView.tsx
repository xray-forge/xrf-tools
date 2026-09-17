import { Box } from "@mui/material";
import { ReactElement, useMemo } from "react";

import { SyntaxContent } from "@/core/syntax/components/SyntaxContent";
import { ESyntaxLanguage } from "@/core/syntax/lib";
import { mergeSx } from "@/core/theme/merge-sx";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface ICodeViewProps extends StyledComponentProps {
  content: string;
  language: ESyntaxLanguage;
  /**
   * Number of the first rendered line.
   *
   * Not always 1: an excerpt lifted out of a file is far more useful when its gutter still says where
   * in that file it came from.
   */
  firstLine?: number;
  label?: string;
}

const CODE_LINE_HEIGHT: number = 1.6;

/**
 * Source text with a line gutter, coloured by its grammar.
 */
export function CodeView({
  "data-testid": dataTestId = "code-view",
  id,
  className,
  content,
  language,
  firstLine = 1,
  label,
  sx,
}: ICodeViewProps): ReactElement {
  const lineNumbers: string = useMemo(() => {
    const count: number = Math.max(1, content.split("\n").length);

    return Array.from({ length: count }, (_, index: number) => firstLine + index).join("\n");
  }, [content, firstLine]);

  return (
    <Box
      data-testid={dataTestId}
      aria-label={label}
      id={id}
      className={className}
      sx={mergeSx({ display: "flex", minWidth: 0, overflow: "auto" }, sx)}
    >
      <Box
        aria-hidden={true}
        component={"pre"}
        className={"monospace m-0 shrink-0 border-r border-divider p-3 text-right text-text-secondary select-none"}
        sx={{ lineHeight: CODE_LINE_HEIGHT }}
      >
        {lineNumbers}
      </Box>

      <Box
        component={"pre"}
        className={"monospace m-0 min-h-full min-w-max p-3 whitespace-pre text-text-primary"}
        sx={{ lineHeight: CODE_LINE_HEIGHT, tabSize: 2 }}
      >
        <SyntaxContent content={content} language={language} />
      </Box>
    </Box>
  );
}
