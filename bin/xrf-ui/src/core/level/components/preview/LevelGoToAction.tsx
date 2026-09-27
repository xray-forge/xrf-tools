import { default as MyLocationIcon } from "@mui/icons-material/MyLocation";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelGoToForm } from "@/core/level/components/preview/LevelGoToForm";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelGoToActionProps extends BaseComponentProps {
  /** Where the camera is as the popover opens, which the fields start at; null before the viewport has drawn. */
  readCamera: () => Nullable<ILevelCamera>;
  isDisabled?: boolean;
  onGoTo: (goTo: ILevelGoTo) => void;
}

/**
 * Stands the camera at a place typed in, in the coordinates the readout shows, or pasted from it.
 */
export function LevelGoToAction({
  "data-testid": dataTestId = "level-go-to-action",
  id,
  className,
  readCamera,
  isDisabled = false,
  onGoTo,
}: ILevelGoToActionProps): ReactElement {
  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Go to"}
      description={"Go to coordinates"}
      icon={<MyLocationIcon />}
      isDisabled={isDisabled}
    >
      <LevelGoToForm readCamera={readCamera} onGoTo={onGoTo} />
    </EditorPopoverAction>
  );
}
