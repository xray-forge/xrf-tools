import { Nullable } from "@/lib/types/general";

export interface ITranslationEntry {
  [language: string]: Nullable<string | Array<string>>;
}

export interface ITranslationJson {
  [entry: string]: ITranslationEntry;
}

export interface ITranslationsProjectJson {
  [file: string]: ITranslationJson;
}
