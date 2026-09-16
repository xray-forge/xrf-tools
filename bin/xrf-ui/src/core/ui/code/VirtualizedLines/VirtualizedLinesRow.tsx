import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { default as WarningAmberIcon } from "@mui/icons-material/WarningAmber";
import { CSSProperties, Fragment, ReactElement, ReactNode } from "react";

import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { CODE } from "@/core/theme/tokens";
import { ECodeLineMark, ICodeLine } from "@/core/ui/code/code-line";

/** Titled so the mark is announced and can be found by name; the icon alone says nothing. */
const MARK_ICONS: Record<ECodeLineMark, ReactNode> = {
  [ECodeLineMark.ERROR]: <ErrorOutlineIcon className={"text-error"} titleAccess={"Error"} />,
  [ECodeLineMark.WARNING]: <WarningAmberIcon className={"text-warning"} titleAccess={"Warning"} />,
};

interface IVirtualizedLinesRowProps {
  line: ICodeLine;
  /** Element id, so the list can point `aria-activedescendant` at the selected line. */
  rowId: string;
  isSelected: boolean;
  /** Measured once by the list, since every gutter in one document has to draw the same column. */
  gutterWidth: number;
  onSelect: (line: ICodeLine) => void;
}

/**
 * One line of a virtualized listing: its gutter cell and its coloured text.
 */
export function VirtualizedLinesRow({
  line,
  rowId,
  isSelected,
  gutterWidth,
  onSelect,
}: IVirtualizedLinesRowProps): ReactElement {
  return (
    <div
      aria-selected={isSelected}
      data-testid={"virtualized-lines-row"}
      id={rowId}
      className={"monospace flex h-code-line w-max min-w-full cursor-default leading-[var(--spacing-code-line)]"}
      role={"option"}
      onClick={() => onSelect(line)}
    >
      <div
        data-selected={isSelected}
        data-testid={"virtualized-lines-gutter"}
        className={
          "sticky left-0 z-1 box-border flex w-[var(--gutter-width)] shrink-0 items-center justify-end gap-1 border-r border-divider surface-content px-2 text-text-secondary select-none data-[selected=true]:text-text-primary [&_svg]:text-[length:var(--gutter-mark-icon)]"
        }
        style={
          {
            "--gutter-mark-icon": `${CODE.markIconSize}px`,
            "--gutter-width": `${gutterWidth}px`,
          } as CSSProperties
        }
      >
        {line.mark ? MARK_ICONS[line.mark] : null}
        {line.number}
      </div>

      <span
        data-selected={isSelected}
        className={"grow px-2.5 whitespace-pre tab-2 data-[selected=true]:bg-action-selected"}
      >
        {line.spans.map((span: ISyntaxSpan, index: number) =>
          span.token === ESyntaxToken.PLAIN ? (
            <Fragment key={index}>{span.text}</Fragment>
          ) : (
            <span key={index} data-syntax-token={span.token}>
              {span.text}
            </span>
          )
        )}
      </span>
    </div>
  );
}
