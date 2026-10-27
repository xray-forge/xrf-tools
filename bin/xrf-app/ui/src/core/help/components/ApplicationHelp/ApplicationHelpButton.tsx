import { HelpOutlineOutlinedIcon as HelpOutlineIcon } from "@xrf/ui/icons";
import { ReactElement } from "react";

import { RailButton } from "@/core/shell/panel/rail/RailButton";

export interface IApplicationHelpButtonProps {
  isDisabled?: boolean;
  onClick: () => void;
}

/**
 * The rail affordance opening the current application's help.
 */
export function ApplicationHelpButton({ isDisabled, onClick }: IApplicationHelpButtonProps): ReactElement {
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
