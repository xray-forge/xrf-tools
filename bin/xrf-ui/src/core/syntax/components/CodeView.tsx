import { ReactElement, useMemo } from "react";

import { SyntaxContent } from "@/core/syntax/components/SyntaxContent";
import { ESyntaxLanguage } from "@/core/syntax/lib";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ICodeViewProps extends BaseComponentProps {
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
}: ICodeViewProps): ReactElement {
  const lineNumbers: string = useMemo(() => {
    const count: number = Math.max(1, content.split("\n").length);

    return Array.from({ length: count }, (_, index: number) => firstLine + index).join("\n");
  }, [content, firstLine]);

  return (
    <div data-testid={dataTestId} aria-label={label} id={id} className={cn("flex min-w-0 overflow-auto", className)}>
      <pre
        aria-hidden={true}
        className={
          "monospace m-0 shrink-0 border-r border-divider p-3 text-right leading-code text-text-secondary select-none"
        }
      >
        {lineNumbers}
      </pre>

      <pre className={"monospace m-0 min-h-full min-w-max p-3 leading-code whitespace-pre tab-2 text-text-primary"}>
        <SyntaxContent content={content} language={language} />
      </pre>
    </div>
  );
}
