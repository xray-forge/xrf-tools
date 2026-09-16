import { default as HelpOutlineIcon } from "@mui/icons-material/HelpOutlineOutlined";
import { ReactElement } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { StyledComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpButtonProps extends StyledComponentProps {
  onClick: () => void;
}

/**
 * The toolbar affordance opening the current application's help.
 */
export function ApplicationHelpButton({
  "data-testid": dataTestId = "application-help-button",
  id,
  className,
  sx,
  onClick,
}: IApplicationHelpButtonProps): ReactElement {
  return (
    <EditorIconAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Help"}
      description={"Help (F1)"}
      icon={<HelpOutlineIcon />}
      sx={sx}
      onClick={onClick}
    />
  );
}
