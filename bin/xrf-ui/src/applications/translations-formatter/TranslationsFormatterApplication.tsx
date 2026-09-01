import { Checkbox, FormControlLabel } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsFormatResult } from "@/applications/translations-formatter/components/TranslationsFormatResult";
import { TranslationsFormatterService } from "@/applications/translations-formatter/services/formatter";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function TranslationsFormatterApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const pathsService: PathsService = useInjection(PathsService);
  const formatterService: TranslationsFormatterService = useInjection(TranslationsFormatterService);

  const [isCheck, setIsCheck] = useState(true);

  // The run rather than this view's own flag: it survives the window being reloaded, so returning here finds it again
  // instead of showing an idle form over sources it is still rewriting.
  const job: Nullable<IJobState> = formatterService.job;
  const isRunning: boolean = Boolean(job);

  const sources: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_FORMATTER,
    id: "sources",
    title: "Select translations sources",
    isDirectory: true,
    isDisabled: isRunning,
    seed: () => resolvePathRole(EPathRole.TRANSLATIONS, pathsService.paths),
  });

  const directory: Nullable<string> = sources.value;

  const onFormat = useCallback(async () => {
    if (!directory) {
      return;
    }

    log.info("Performing translations format command:", isCheck, directory);

    await formatterService.format(directory, isCheck);
  }, [directory, formatterService, isCheck, log]);

  const onCancel = useCallback(() => formatterService.cancel(), [formatterService]);

  const onCheckModeChange = useCallback(
    (_: ChangeEvent<HTMLInputElement>, checked: boolean) => {
      formatterService.reset();
      setIsCheck(checked);
    },
    [formatterService]
  );

  useEffect(() => {
    formatterService.reset();
  }, [directory, formatterService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!sources.isValid}
      title={isCheck ? "Check translations formatting" : "Format translation sources"}
      description={
        isCheck
          ? "Reports which sources are not normalized. Nothing is written."
          : "Rewrites every unformatted source in the directory in place."
      }
      error={formatterService.error ?? undefined}
      submitLabel={isCheck ? "Check" : "Format"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        formatterService.result ? <TranslationsFormatResult isCheck={isCheck} result={formatterService.result} /> : null
      }
      onSubmit={onFormat}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Translations directory"}
        description={"Directory of JSON translation sources to format"}
        field={sources}
      />

      <FormControlLabel
        control={<Checkbox disabled={isRunning} checked={isCheck} onChange={onCheckModeChange} />}
        label={"Check only"}
      />
    </PickerForm>
  );
}
