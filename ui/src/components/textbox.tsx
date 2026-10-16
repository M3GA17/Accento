'use client';
import React, { forwardRef, useRef, useState, useImperativeHandle, useId } from 'react';
import { TextField, Input, Label, Text, FieldError } from 'react-aria-components';
import { Button } from './button';
import { defaultField, useFormReset, type FieldPresentation } from './field';

export type TextBoxType = 'text' | 'email' | 'password' | 'tel' | 'url' | 'number' | 'search';
export type TextBoxProps = Omit<React.InputHTMLAttributes<HTMLInputElement>,
  'size' | 'value' | 'defaultValue' | 'children' | 'type' | 'className'> & FieldPresentation & {
  type?: TextBoxType; label?: string; value?: string; defaultValue?: string;
  onValueChange?: (value: string) => void; description?: string; error?: string;
  showCharacterCount?: boolean; revealPassword?: boolean; revealContent?: React.ReactNode;
  concealContent?: React.ReactNode; className?: string;
};
export const TextBox = forwardRef<HTMLInputElement, TextBoxProps>(function TextBox(
  { label, value, defaultValue = '', onValueChange, description, error, showCharacterCount = false,
    revealPassword = false, revealContent = 'Mostra', concealContent = 'Nascondi', disabled = false,
    readOnly = false, required = false, type = 'text', render = defaultField, className = '',
    id, name, 'aria-label': accessibleName, 'aria-invalid': invalid, ...inputProps }, forwardedRef,
) {
  const inputRef = useRef<HTMLInputElement>(null);
  const counterId = useId();
  useImperativeHandle(forwardedRef, () => inputRef.current!);
  const [internalValue, setInternalValue] = useState(defaultValue);
  const [visible, setVisible] = useState(false);
  const current = value ?? internalValue;
  useFormReset(inputRef, value === undefined, () => setInternalValue(defaultValue));
  return <TextField id={id} name={name} value={current}
    onChange={next => { setInternalValue(next); onValueChange?.(next); }}
    isDisabled={disabled} isReadOnly={readOnly} isRequired={required}
    isInvalid={!!error || invalid === true || invalid === 'true'} validationBehavior="native"
    aria-label={accessibleName} className={'accento-field ' + className} data-accento-component="textbox" data-filled={!!current || undefined}
    data-has-label={!!label || undefined} type={type === 'password' && visible ? 'text' : type}>
    {render({
      label: label ? <Label data-accento-part="label">{label}{required && <span aria-hidden="true"> *</span>}</Label> : null,
      input: <Input {...inputProps} ref={inputRef} data-accento-part="input" aria-describedby={[inputProps['aria-describedby'], showCharacterCount ? counterId : null].filter(Boolean).join(' ') || undefined}
        onChange={inputProps.onChange} />,
      value: current,
      description: description && !error ? <Text slot="description" data-accento-part="description">{description}</Text> : null,
      error: <FieldError data-accento-part="error">{error}</FieldError>,
      counter: showCharacterCount ? <span id={counterId} data-accento-part="counter">{current.length}{inputProps.maxLength !== undefined ? ' / ' + inputProps.maxLength : ''}</span> : null,
      action: type === 'password' && revealPassword ? <Button slot={null} disabled={disabled}
        aria-label={visible ? 'Nascondi password' : 'Mostra password'} aria-pressed={visible}
        data-accento-part="field-action" onPress={() => setVisible(v => !v)}>{visible ? concealContent : revealContent}</Button> : null,
    })}
  </TextField>;
});
