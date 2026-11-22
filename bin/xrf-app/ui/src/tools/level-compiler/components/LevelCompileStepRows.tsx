import { ELevelCompileStep, LevelCompileStep } from "@xrf/ipc/types/xrf-level-compiler";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ReactElement } from "react";

import { COMPILE_STEP_LABELS } from "@/tools/level-compiler/lib/compile-options";
import { COMPILE_STEPS, toggleCompileStep } from "@/tools/level-compiler/lib/compile-request";

interface ILevelCompileStepRowsProps {
  steps: ReadonlySet<LevelCompileStep>;
  /** Whether the scene holds details; null while unknown, which offers the step. */
  hasDetails: Nullable<boolean>;
  isDisabled: boolean;
  onChange: (steps: ReadonlySet<LevelCompileStep>) => void;
}

/** Why the details box is disabled for a scene without details. */
const NO_DETAILS_DESCRIPTION: string = "The scene holds no details, so its build writes no build.details to light";

/**
 * One box per compile step, in run order. The light box is disabled without the geometry it is baked with, and ticking
 * either changes both as {@link toggleCompileStep} decides; the details box is disabled for a scene without details.
 */
export function LevelCompileStepRows({
  steps,
  hasDetails,
  isDisabled,
  onChange,
}: ILevelCompileStepRowsProps): ReactElement {
  return (
    <>
      {COMPILE_STEPS.map((step: LevelCompileStep) => {
        const isWithoutDetails: boolean = step === ELevelCompileStep.DETAILS && hasDetails === false;

        return (
          <CheckboxFormRow
            key={step}
            label={COMPILE_STEP_LABELS[step].label}
            description={isWithoutDetails ? NO_DETAILS_DESCRIPTION : COMPILE_STEP_LABELS[step].description}
            isChecked={steps.has(step)}
            isDisabled={
              isDisabled ||
              isWithoutDetails ||
              (step === ELevelCompileStep.LIGHT && !steps.has(ELevelCompileStep.GEOMETRY))
            }
            onChange={(isChecked: boolean) => onChange(toggleCompileStep(steps, step, isChecked))}
          />
        );
      })}
    </>
  );
}
