import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { ConfigsDeclarationSearch } from "@xrf/ipc/types/xrf-app-configs";
import { LtxDeclaration } from "@xrf/ipc/types/xrf-ltx-inspect";
import { Nullable } from "@xrf/types";
import { Button } from "@xrf/ui/button";
import { EditorFilterInput } from "@xrf/ui/editor/filter-input";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { MyLocationIcon } from "@xrf/ui/icons/my-location";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { ErrorState } from "@xrf/ui/layout/error-state";
import { List } from "@xrf/ui/list";
import { KeyboardEvent, ReactElement, useCallback } from "react";

import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import { inline } from "@/lib/callbacks/inline";
import { ConfigsDocumentService } from "@/tools/configs-explorer/services/configs-document";
import { ConfigsFindService, IConfigsFindQuery } from "@/tools/configs-explorer/services/configs-find";

import { ConfigsFindRow } from "./ConfigsFindRow";

/**
 * Where the open project writes a section, a field or a value, each row opening the config at its line.
 *
 * Rows show what is written, not resolved; the Resolved view says which write the engine ends up with.
 */
export function ConfigsFindPanel({
  "data-testid": dataTestId = "configs-find-panel",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const findService: ConfigsFindService = useInjection(ConfigsFindService);

  const query: IConfigsFindQuery = findService.query;
  const found: Nullable<ConfigsDeclarationSearch> = findService.results.value;
  const selectedSection: Nullable<string> = documentService.selectedSection;

  const onFind = useCallback(() => void findService.find(), [findService]);

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      if (event.key === "Enter") {
        void findService.find();
      }
    },
    [findService]
  );

  const onSectionChange = useCallback((section: string) => findService.setQuery({ section }), [findService]);
  const onFieldChange = useCallback((field: string) => findService.setQuery({ field }), [findService]);
  const onValueChange = useCallback((value: string) => findService.setQuery({ value }), [findService]);

  // Every config declaring or patching the selected section, archived ones included.
  const onFindSelected = useCallback(() => {
    if (selectedSection) {
      void findService.findSection(selectedSection);
    }
  }, [findService, selectedSection]);

  const onOpen = useCallback(
    (declaration: LtxDeclaration) => void documentService.openAt(declaration.path, declaration.line),
    [documentService]
  );

  return (
    <EditorPanel
      data-testid={dataTestId}
      title={"Find"}
      actions={
        <EditorIconAction
          label={"Find the selected section"}
          description={selectedSection ? `Find every config writing [${selectedSection}]` : "Select a section first"}
          icon={<MyLocationIcon />}
          isDisabled={!selectedSection}
          onClick={onFindSelected}
        />
      }
    >
      <div className={"flex flex-col gap-1.5 p-2"}>
        <EditorFilterInput
          ariaLabel={"Section"}
          placeholder={"Section"}
          query={query.section}
          onKeyDown={onKeyDown}
          onQueryChange={onSectionChange}
        />
        <EditorFilterInput
          ariaLabel={"Field"}
          placeholder={"Field"}
          query={query.field}
          onKeyDown={onKeyDown}
          onQueryChange={onFieldChange}
        />
        <EditorFilterInput
          ariaLabel={"Value contains"}
          placeholder={"Value contains"}
          query={query.value}
          onKeyDown={onKeyDown}
          onQueryChange={onValueChange}
        />
        <Button size={"small"} variant={"outlined"} onClick={onFind}>
          Find
        </Button>
      </div>

      {inline(() => {
        if (findService.results.isFailed) {
          return (
            <ErrorState
              title={"Cannot search the configs"}
              description={findService.results.error?.message ?? "The search failed."}
              onRetry={onFind}
            />
          );
        }

        if (findService.results.isLoading) {
          return <DelayedProgress />;
        }

        if (!found) {
          return <EditorPanelEmpty label={"Name a section, a field or a value, then press Enter."} />;
        }

        if (!found.search.declarations.length) {
          return <EditorPanelEmpty label={"Nothing in this project writes that."} />;
        }

        return (
          <>
            {found.total > found.search.declarations.length ? (
              <EditorPanelEmpty
                data-testid={"configs-find-truncated"}
                label={`Showing the first ${formatCount(found.search.declarations.length)} of ${formatCount(found.total)}; narrow the search to see the rest.`}
              />
            ) : null}

            {found.search.unreadable.length ? (
              <EditorPanelEmpty
                label={`${formatCount(found.search.unreadable.length)} configs do not parse and were not searched.`}
              />
            ) : null}

            <List dense disablePadding aria-label={"Found statements"}>
              {found.search.declarations.map((declaration: LtxDeclaration) => (
                <ConfigsFindRow
                  key={`${declaration.path}:${declaration.line}`}
                  declaration={declaration}
                  onOpen={onOpen}
                />
              ))}
            </List>
          </>
        );
      })}
    </EditorPanel>
  );
}
