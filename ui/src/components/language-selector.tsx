'use client';
import React, {forwardRef, useEffect, useRef, useState} from 'react';
import {Autocomplete, Button, Input, ListBox, ListBoxItem, Popover, SearchField, Select, SelectValue, Text, useLocale} from 'react-aria-components';
import {useFormReset} from './field';

export interface LanguageOption {
  /** Unique locale identifier, e.g. en-GB. The consumer owns the supported locales. */
  locale: string;
  label: string;
  code?: string;
  description?: string;
  keywords?: readonly string[];
  /** URL of a flag asset. Optional: languages and countries are separate identities. */
  flagSrc?: string;
  disabled?: boolean;
}
export interface LanguageSelectorParts {
  trigger: React.ReactNode;
  popup: React.ReactNode;
  selected: LanguageOption | undefined;
  open: boolean;
}
export interface LanguageSelectorProps {
  label: string;
  options: readonly LanguageOption[];
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (locale: string) => void;
  id?: string;
  name?: string;
  disabled?: boolean;
  loading?: boolean;
  display?: 'code' | 'name';
  searchable?: boolean;
  placeholder?: string;
  searchLabel?: string;
  searchPlaceholder?: string;
  clearSearchLabel?: string;
  emptyMessage?: string;
  loadingLabel?: string;
  dir?: 'ltr' | 'rtl';
  className?: string;
  popupClassName?: string;
  render?: (parts: LanguageSelectorParts) => React.ReactNode;
  indicator?: React.ReactNode;
  selectionIndicator?: React.ReactNode;
  searchIcon?: React.ReactNode;
  clearIcon?: React.ReactNode;
  fallbackIcon?: React.ReactNode;
}

function Flag({src, fallback, size}: {src?: string; fallback: React.ReactNode; size: number}) {
  const [failed, setFailed] = useState<string>();
  return <span data-accento-part="flag" aria-hidden="true">{src && failed !== src ?
    <img src={src} alt="" width={size} height={size} onError={() => setFailed(src)}/> : fallback}</span>;
}
const normalize = (value: string) => value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();

/** Select + optional Autocomplete share the collection, selection, form and keyboard state. */
export const LanguageSelector = forwardRef<HTMLButtonElement, LanguageSelectorProps>(function LanguageSelector({
  label, options, value, defaultValue = null, onValueChange, id, name, disabled = false, loading = false,
  display = 'code', searchable = options.length > 8, placeholder = 'Lingua',
  searchLabel = 'Cerca lingua', searchPlaceholder = 'Cerca lingua o paese…', clearSearchLabel = 'Cancella ricerca',
  emptyMessage = 'Nessuna lingua trovata', loadingLabel = 'Caricamento delle lingue…', dir,
  className = '', popupClassName = '', render, indicator = <span aria-hidden="true">⌄</span>, selectionIndicator = '✓', searchIcon,
  clearIcon = '×', fallbackIcon = <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4" aria-hidden="true"><circle cx="12" cy="12" r="9"/><ellipse cx="12" cy="12" rx="4" ry="9"/><path d="M3 12h18"/></svg>,
}, ref) {
  const root = useRef<HTMLDivElement>(null);
  const {direction} = useLocale();
  const resetting = useRef(false);
  const [internal, setInternal] = useState(defaultValue);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const selectedValue = value === undefined ? internal : value;
  const selected = options.find(option => option.locale === selectedValue);
  useEffect(() => {if (disabled || loading) {setOpen(false); setQuery('');}}, [disabled, loading]);
  // The hidden native select also handles reset. Only our uncontrolled reset may change the value.
  useEffect(() => {
    const form = root.current?.closest('form');
    if (!form) return;
    const reset = () => {resetting.current = true; setTimeout(() => {resetting.current = false;}, 0);};
    form.addEventListener('reset', reset, true);
    return () => form.removeEventListener('reset', reset, true);
  });
  useFormReset(root, value === undefined, () => {setInternal(defaultValue); setOpen(false); setQuery('');});
  const flag = (option: LanguageOption | undefined, size: number) => <Flag src={option?.flagSrc} fallback={fallbackIcon} size={size}/>;
  const list = <ListBox<LanguageOption> items={options} aria-label={label} data-accento-part="list"
    renderEmptyState={() => <div data-accento-part="empty" role="status">{emptyMessage}</div>}>
    {option => <ListBoxItem id={option.locale} textValue={option.label} isDisabled={option.disabled || loading} data-accento-part="option"
      data-has-description={!!option.description || undefined}>
      {({isSelected}) => <>{flag(option, 18)}<Text slot="label" data-accento-part="option-copy">
        <bdi data-accento-part="option-label">{option.label}</bdi>
        {option.description && <> <small data-accento-part="option-description"><bdi>{option.description}</bdi></small></>}
      </Text><span data-accento-part="selection-indicator" aria-hidden="true">{isSelected && selectionIndicator}</span></>}
    </ListBoxItem>}
  </ListBox>;
  const popup = <Popover placement={dir && dir !== direction ? 'bottom end' : 'bottom start'} offset={4} className={'accento-language-popup ' + popupClassName} dir={dir}
    data-accento-component="language-popup" data-searchable={searchable || undefined} data-loading={loading || undefined}
    render={domProps => <div {...domProps} dir={dir ?? domProps.dir} aria-busy={loading || undefined}/> }>
    {loading && <div role="status" data-accento-part="loading">{loadingLabel}</div>}
    {searchable ? <Autocomplete<LanguageOption> inputValue={query} onInputChange={setQuery}
      filter={(_text, input, node) => normalize([node.value?.label, node.value?.locale, node.value?.code,
        node.value?.description, ...(node.value?.keywords ?? [])].join(' ')).includes(normalize(input.trim()))}>
      <SearchField aria-label={searchLabel} autoFocus data-accento-part="search" isDisabled={loading}>
        {searchIcon}<Input placeholder={searchPlaceholder} data-accento-part="search-input"/>
        <Button aria-label={clearSearchLabel} data-accento-part="search-clear">{clearIcon}</Button>
      </SearchField><div data-accento-part="list-scroll" tabIndex={0} role="group" aria-label={label}>{list}</div>
    </Autocomplete> : list}
  </Popover>;
  const trigger = <Button ref={ref} data-accento-part="trigger" aria-label={label + ': ' + (selected ? [selected.label, selected.description].filter(Boolean).join(', ') : placeholder) + (loading ? ', ' + loadingLabel : '')}
    render={domProps => <button {...domProps} aria-busy={loading || undefined}/> }>
    <SelectValue<LanguageOption> data-accento-part="value">{() => <>{flag(selected, 22)}
      <bdi data-accento-part="value-label">{selected ? display === 'name' ? selected.label : selected.code ?? selected.locale.toUpperCase() : placeholder}</bdi>
    </>}</SelectValue><span data-accento-part="indicator" aria-hidden="true">{indicator}</span>
  </Button>;
  const parts = {trigger, popup, selected, open};
  return <Select<LanguageOption> ref={root} id={id} name={name} aria-label={label} dir={dir} isDisabled={disabled || loading}
    disabledKeys={options.filter(option => option.disabled).map(option => option.locale)}
    value={selectedValue} onChange={key => {
      const option = options.find(option => option.locale === key);
      if (!option || option.disabled || loading || disabled || resetting.current || option.locale === selectedValue) return;
      if (value === undefined) setInternal(option.locale);
      onValueChange?.(option.locale);
    }} isOpen={open && !disabled && !loading} onOpenChange={next => {setOpen(next); setQuery('');}}
    className={'accento-language ' + className} data-accento-component="language-selector" data-display={display}
    data-loading={loading || undefined}>
    {render ? render(parts) : <>{trigger}{popup}</>}
  </Select>;
});
