import { ELevelCompileStep, LevelCompileStep } from "@xrf/ipc/types/xrf-level-compiler";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ReactElement } from "react";

import { COMPILE_STEP_LABELS } from "@/tools/level-compiler/lib/compile-options";
import { COMPILE_STEPS, toggleCompileStep } from "@/tools/level-compiler/lib/compile-request";

interface ILevelCompileStepRowsProps {
  steps: ReadonlySet<LevelCompileStep>;
  isDisabled: boolean;
  onChange: (steps: ReadonlySet<LevelCompileStep>) => void;
}

/**
 * One box per compile step, in run order. The light box is disabled without the geometry it is baked with, and ticking
 * either changes both as {@link toggleCompileStep} decides.
 */
export function LevelCompileStepRows({ steps, isDisabled, onChange }: ILevelCompileStepRowsProps): ReactElement {
  return (
    <>
      {COMPILE_STEPS.map((step: LevelCompileStep) => (
        <CheckboxFormRow
          key={step}
          label={COMPILE_STEP_LABELS[step].label}
          description={COMPILE_STEP_LABELS[step].description}
          isChecked={steps.has(step)}
          isDisabled={isDisabled || (step === ELevelCompileStep.LIGHT && !steps.has(ELevelCompileStep.GEOMETRY))}
          onChange={(isChecked: boolean) => onChange(toggleCompileStep(steps, step, isChecked))}
        />
      ))}
    </>
  );
}
