import { ReactElement, useMemo } from "react";

import { IEngineChoice, toAutoEngineLabel } from "@/core/engine-target/lib";
import { EXrayEngineChoice } from "@/core/ipc/types/xrf-engine-target";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form";

interface IEngineChoiceFormRowProps {
  engine: IEngineChoice;
  isDisabled?: boolean;
}

/** Choose which engine a game root is read as: what its files show, or one named over them. */
export function EngineChoiceFormRow({ engine, isDisabled }: IEngineChoiceFormRowProps): ReactElement {
  const auto: string = toAutoEngineLabel(engine.detection, engine.isDetecting);

  const options: ReadonlyArray<IChoiceFormRowOption<EXrayEngineChoice>> = useMemo(
    () => [
      { "aria-label": auto, label: "Auto", value: EXrayEngineChoice.AUTO },
      { label: "Vanilla", value: EXrayEngineChoice.VANILLA },
      { label: "Extended", value: EXrayEngineChoice.EXTENDED },
    ],
    [auto]
  );

  return (
    <ChoiceFormRow
      label={"Engine"}
      description={`${auto}. Vanilla: OpenXRay and Call of Pripyat; Extended: Anomaly. Remembered per root.`}
      options={options}
      value={engine.choice}
      isDisabled={isDisabled}
      onChange={engine.setChoice}
    />
  );
}
