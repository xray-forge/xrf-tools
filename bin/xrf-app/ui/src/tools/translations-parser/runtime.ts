import { IContainerDefinition } from "@/lib/container/container-definition";
import { TranslationsParserService } from "@/tools/translations-parser/services/translations-parser.service";

export { TranslationsParserTool as Component } from "./TranslationsParserTool";

export const container: IContainerDefinition = { bindings: [TranslationsParserService] };
