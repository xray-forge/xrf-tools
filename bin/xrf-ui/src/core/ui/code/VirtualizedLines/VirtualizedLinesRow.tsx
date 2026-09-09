import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { default as WarningAmberIcon } from "@mui/icons-material/WarningAmber";
import { Box } from "@mui/material";
import { Fragment, ReactElement, ReactNode } from "react";

import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { CODE, MONOSPACE } from "@/core/theme/tokens";
import { ECodeLineMark, ICodeLine } from "@/core/ui/code/code-line";

/** Titled so the mark is announced and can be found by name; the icon alone says nothing. */
const MARK_ICONS: Record<ECodeLineMark, ReactNode> = {
  [ECodeLineMark.ERROR]: <ErrorOutlineIcon sx={{ color: "error.main" }} titleAccess={"Error"} />,
  [ECodeLineMark.WARNING]: <WarningAmberIcon sx={{ color: "warning.main" }} titleAccess={"Warning"} />,
};

interface IVirtualizedLinesRowProps {
  line: ICodeLine;
  /** Element id, so the list can point `aria-activedescendant` at the selected line. */
  rowId: string;
  isSelected: boolean;
  /** Measured once by the list, since every gutter in one document has to draw the same column. */
  gutterWidth: number;
  /** Resolved once by the list rather than per row, which would read the theme thousands of times. */
  colors: Record<ESyntaxToken, string>;
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
  colors,
  onSelect,
}: IVirtualizedLinesRowProps): ReactElement {
  return (
    <Box
      aria-selected={isSelected}
      data-testid={"virtualized-lines-row"}
      id={rowId}
      role={"option"}
      sx={{
        cursor: "default",
        display: "flex",
        fontFamily: MONOSPACE.fontFamily,
        fontSize: MONOSPACE.fontSize,
        height: CODE.lineHeight,
        lineHeight: `${CODE.lineHeight}px`,
        // A row is as wide as its content and never narrower than the viewport, so a long value
        // scrolls sideways while a short line still draws its selection across the whole width.
        minWidth: "100%",
        width: "max-content",
      }}
      onClick={() => onSelect(line)}
    >
      <Box
        data-testid={"virtualized-lines-gutter"}
        sx={{
          alignItems: "center",
          // Opaque, and pinned to the left edge of the scroller: the text passes underneath it.
          backgroundColor: "background.default",
          borderColor: "divider",
          borderRight: 1,
          boxSizing: "border-box",
          // The gutter keeps the page colour when the line is selected and brightens its number
          // instead, since a translucent selection here would let the text show through it.
          color: isSelected ? "text.primary" : "text.secondary",
          display: "flex",
          flexShrink: 0,
          gap: `${CODE.gutterGap}px`,
          justifyContent: "flex-end",
          left: 0,
          paddingX: `${CODE.gutterPaddingX}px`,
          position: "sticky",
          userSelect: "none",
          width: gutterWidth,
          zIndex: 1,
          "& svg": { fontSize: CODE.markIconSize },
        }}
      >
        {line.mark ? MARK_ICONS[line.mark] : null}
        {line.number}
      </Box>

      <Box
        component={"span"}
        sx={{
          backgroundColor: isSelected ? "action.selected" : "transparent",
          flexGrow: 1,
          paddingX: `${CODE.contentPaddingX}px`,
          tabSize: 2,
          whiteSpace: "pre",
        }}
      >
        {line.spans.map((span: ISyntaxSpan, index: number) =>
          span.token === ESyntaxToken.PLAIN ? (
            <Fragment key={index}>{span.text}</Fragment>
          ) : (
            <span key={index} style={{ color: colors[span.token] }}>
              {span.text}
            </span>
          )
        )}
      </Box>
    </Box>
  );
}
