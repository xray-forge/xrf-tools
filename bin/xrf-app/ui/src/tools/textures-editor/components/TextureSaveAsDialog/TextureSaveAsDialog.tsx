import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import {
  ETextureReferenceField,
  ETextureSaveAsBump,
  ETextureSaveAsRole,
  TextureReferenceField,
  TextureReferenceRename,
  TextureSaveAsBump,
  TextureSaveAsFile,
  TextureSaveAsPlan,
  TextureSaveAsRole,
} from "@xrf/ipc/types/xrf-app-textures";
import { assertExhaustive, Nullable } from "@xrf/types";
import { Button } from "@xrf/ui/button";
import { DialogHeader } from "@xrf/ui/dialog/header";
import { Dialog, DialogActions, DialogContent } from "@xrf/ui/dialog/modal";
import { Accordion, AccordionDetails, AccordionSummary } from "@xrf/ui/disclosure";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Alert } from "@xrf/ui/feedback";
import { ChoiceFormRow, IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { FormRow } from "@xrf/ui/form/row";
import { ExpandMoreIcon } from "@xrf/ui/icons/expand-more";
import { TextField } from "@xrf/ui/input";
import { cn } from "@xrf/ui/theme";
import { Typography } from "@xrf/ui/typography";
import { ChangeEvent, ReactElement, useCallback, useEffect, useId, useState } from "react";

import { ReplaceConfirmDialog } from "@/core/jobs/components/ReplaceConfirmDialog";
import { IReplaceRetry, IReplaceRetryStart, useReplaceRetry } from "@/core/jobs/lib/use-replace-retry";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { ITextureSaveAs, TextureSaveAsService } from "@/tools/textures-editor/services/texture-save-as";

/** Appended to the original's name to suggest the copy's name. */
const COPY_SUFFIX: string = "_copy";

/** How long typing must pause before the name is checked, as a path field waits. */
const REVIEW_DEBOUNCE_MS: number = 250;

/** The two answers to what the copy does with the bump map, the self-contained one first. */
const BUMP_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TextureSaveAsBump>> = [
  { value: ETextureSaveAsBump.OWN, label: "Copy it" },
  { value: ETextureSaveAsBump.SHARED, label: "Share the original's" },
];

/**
 * @param role - Which of the texture's files is written.
 * @returns Its label in the list of files to write.
 */
function getRoleLabel(role: TextureSaveAsRole): string {
  switch (role) {
    case ETextureSaveAsRole.TEXTURE:
      return "Texture";
    case ETextureSaveAsRole.DESCRIPTOR:
      return "Descriptor";
    case ETextureSaveAsRole.BUMP:
      return "Bump map";
    case ETextureSaveAsRole.COMPANION:
      return "Bump map companion";
    case ETextureSaveAsRole.BUMP_DESCRIPTOR:
      return "Bump map descriptor";
    default:
      return assertExhaustive(role);
  }
}

/**
 * @param field - A descriptor field that names a texture.
 * @returns Its label in the descriptor form.
 */
function getFieldLabel(field: TextureReferenceField): string {
  switch (field) {
    case ETextureReferenceField.BUMP:
      return "Bump texture";
    case ETextureReferenceField.DETAIL:
      return "Detail texture";
    default:
      return assertExhaustive(field);
  }
}

/**
 * @param bump - What the copy does with the bump map.
 * @returns What that choice means for the copy, in one sentence.
 */
function describeBumpChoice(bump: TextureSaveAsBump): string {
  switch (bump) {
    case ETextureSaveAsBump.OWN:
      return "The bump map files are copied under the new name, and the copy uses them.";
    case ETextureSaveAsBump.SHARED:
      return "No bump map files are copied; the copy uses the original's, so a change to them shows on both.";
    default:
      return assertExhaustive(bump);
  }
}

/**
 * @param request - The copy's name and bump map choice.
 * @returns One key for both, so a change to either is noticed.
 */
function toReviewKey(request: ITextureSaveAs): string {
  return `${request.bump}:${request.destination}`;
}

interface ITextureSaveAsDialogProps extends BaseComponentProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Saves a copy of the open texture under a new name: shows the files it writes, then writes the copy and closes. Asks
 * before replacing files that already have the name. What happens to the bump map is under "Advanced".
 */
export function TextureSaveAsDialog({
  "data-testid": dataTestId = "texture-save-as-dialog",
  id,
  className,
  isOpen,
  onClose,
}: ITextureSaveAsDialogProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const saveAsService: TextureSaveAsService = useInjection(TextureSaveAsService);
  const titleId: string = useId();

  const reference: string = selectionService.reference ?? "";
  const [destination, setDestination] = useState<string>("");
  const [bump, setBump] = useState<TextureSaveAsBump>(ETextureSaveAsBump.OWN);
  // The name and choice last sent for checking, which lag the typed name until typing pauses.
  const [reviewed, setReviewed] = useState<string>("");

  const request: ITextureSaveAs = { destination, bump };
  const plan: Nullable<TextureSaveAsPlan> = saveAsService.plan.value;
  const planError: Nullable<string> = saveAsService.plan.error?.message ?? null;
  const isWriting: boolean = saveAsService.saveAs.isRunning;
  // An out-of-date plan is greyed rather than removed, so the dialog keeps its size while a name is typed.
  const isPlanCurrent: boolean = reviewed === toReviewKey(request) && saveAsService.plan.isReady;

  // The replace question is asked after this dialog closes, so it lives here rather than in the dialog.
  const save: IReplaceRetry<ITextureSaveAs> = useReplaceRetry(
    useCallback(
      ({ request, isForced }: IReplaceRetryStart<ITextureSaveAs>) => flowResult(saveAsService.run(request, isForced)),
      [saveAsService]
    )
  );

  const review = useCallback(
    (next: ITextureSaveAs) => {
      setReviewed(toReviewKey(next));
      void saveAsService.review(next);
    },
    [saveAsService]
  );

  // A choice is checked at once; only typing waits.
  const onBumpChange = useCallback(
    (next: TextureSaveAsBump) => {
      setBump(next);
      review({ destination, bump: next });
    },
    [destination, review]
  );

  const onSave = useCallback(async (): Promise<void> => {
    onClose();

    // The job's notification reports the result; a name already in use opens the replace prompt.
    await save.run({ destination, bump });
  }, [bump, destination, onClose, save]);

  // Each opening suggests a name, copies the bump map, and checks at once.
  useEffect(() => {
    if (isOpen) {
      const initial: ITextureSaveAs = { destination: `${reference}${COPY_SUFFIX}`, bump: ETextureSaveAsBump.OWN };

      setDestination(initial.destination);
      setBump(initial.bump);
      review(initial);
    }
  }, [isOpen, reference, review]);

  // A typed name is checked once typing pauses, not on every key.
  useEffect(() => {
    if (!isOpen || reviewed === toReviewKey({ destination, bump })) {
      return;
    }

    const handle = setTimeout(() => review({ destination, bump }), REVIEW_DEBOUNCE_MS);

    return () => clearTimeout(handle);
  }, [bump, destination, isOpen, review, reviewed]);

  return (
    <>
      <ReplaceConfirmDialog retry={save} />

      <Dialog
        data-testid={dataTestId}
        id={id}
        className={className}
        aria-labelledby={titleId}
        fullWidth
        maxWidth={"sm"}
        open={isOpen}
        onClose={onClose}
      >
        <DialogHeader title={"Save as"} titleId={titleId} closeLabel={"Close save as"} onClose={onClose} />

        <DialogContent className={"flex flex-col gap-4"}>
          <FormRow
            label={"Name"}
            description={`Texture name for the copy, written like ${reference}. The original is not changed.`}
            error={planError}
          >
            {(control) => (
              <TextField
                {...control}
                size={"small"}
                fullWidth
                disabled={isWriting}
                value={destination}
                onChange={(event: ChangeEvent<HTMLInputElement>) => setDestination(event.target.value)}
              />
            )}
          </FormRow>

          {plan ? (
            <div data-testid={"texture-save-as-files"} className={cn(!isPlanCurrent && "opacity-60")}>
              <Typography variant={"subtitle2"}>Files to write</Typography>

              {plan.files.map((file: TextureSaveAsFile) => (
                <Typography key={file.to} className={"block font-monospace wrap-anywhere"} variant={"caption"}>
                  {getRoleLabel(file.role)}: {file.to}
                  {file.isReplacing ? " (replaces an existing file)" : ""}
                </Typography>
              ))}
            </div>
          ) : null}

          {plan?.hasBumpPair ? (
            <Accordion data-testid={"texture-save-as-advanced"} disableGutters variant={"outlined"}>
              <AccordionSummary expandIcon={<ExpandMoreIcon fontSize={"small"} />}>
                <Typography variant={"body2"}>Advanced</Typography>
              </AccordionSummary>

              <AccordionDetails className={cn("flex flex-col gap-2", !isPlanCurrent && "opacity-60")}>
                <ChoiceFormRow
                  data-testid={"texture-save-as-bump"}
                  label={"Bump map"}
                  description={describeBumpChoice(bump)}
                  options={BUMP_OPTIONS}
                  value={bump}
                  isDisabled={isWriting}
                  onChange={onBumpChange}
                />

                {plan.renames.length ? (
                  <div data-testid={"texture-save-as-renames"}>
                    <Typography className={"text-text-secondary"} variant={"caption"}>
                      Changed in the copy&apos;s descriptor:
                    </Typography>

                    {plan.renames.map((rename: TextureReferenceRename) => (
                      <Typography
                        key={rename.field}
                        data-testid={"texture-save-as-rename-field"}
                        className={"block font-monospace wrap-anywhere"}
                        variant={"caption"}
                      >
                        {getFieldLabel(rename.field)}: {rename.from} → {rename.to}
                      </Typography>
                    ))}
                  </div>
                ) : null}
              </AccordionDetails>
            </Accordion>
          ) : null}

          {saveAsService.saveAs.error ? <Alert severity={"error"}>{saveAsService.saveAs.error}</Alert> : null}
        </DialogContent>

        <DialogActions>
          <Button size={"small"} onClick={onClose}>
            Cancel
          </Button>

          <Button
            data-testid={"texture-save-as-confirm"}
            size={"small"}
            variant={"contained"}
            disabled={!saveAsService.canSaveAs || !isPlanCurrent}
            onClick={() => void onSave()}
          >
            Save as
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
}
