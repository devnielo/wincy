import en from "./en";
import es from "./es";

export const dictionaries = { en, es };
export type Lang = keyof typeof dictionaries;
