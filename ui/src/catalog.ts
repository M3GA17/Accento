import type {LanguageOption} from '@dimalabs/react-component-flat-product';
import languages from '../../localization/languages.json';
export const interfaceLanguages:LanguageOption[] = languages.map(language=>({
  locale:language.code,label:language.name,code:language.code.toUpperCase(),keywords:[language.english,...language.countries],
}));
export const characterSets = languages;
