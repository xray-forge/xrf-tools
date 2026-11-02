import { HelpOutlineOutlinedIcon as HelpOutlineIcon } from "@xrf/ui/icons/help-outline-outlined";
import { RailButton } from "@xrf/ui/shell/rail-button";
import { ReactElement } from "react";

export interface IToolHelpButtonProps {
  isDisabled?: boolean;
  onClick: () => void;
}

/**
 * The rail affordance opening the current tool's help.
 */
export function ToolHelpButton({ isDisabled, onClick }: IToolHelpButtonProps): ReactElement {
  return (
    <RailButton
      label={"Help"}
      description={isDisabled ? "No help written for this screen yet" : "Help (F1)"}
      icon={<HelpOutlineIcon />}
      isDisabled={isDisabled}
      onClick={onClick}
    />
  );
}
