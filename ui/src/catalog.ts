import type {LanguageOption} from './components';
import languages from '../../localization/languages.json';
export const interfaceLanguages:LanguageOption[] = languages.map(language=>({
  locale:language.code,label:language.name,code:language.code.toUpperCase(),keywords:[language.english,...language.countries],
}));
export const characterSets = languages;
