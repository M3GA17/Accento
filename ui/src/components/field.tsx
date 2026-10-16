'use client';
import React, { useEffect, type RefObject } from 'react';

export interface FieldParts {
  label: React.ReactNode;
  input: React.ReactNode;
  description: React.ReactNode;
  error: React.ReactNode;
  counter?: React.ReactNode;
  action?: React.ReactNode;
  value: string;
}
export interface FieldPresentation {
  render?: (parts: FieldParts) => React.ReactNode;
}
export function defaultField(parts: FieldParts) {
  return <>{parts.label}<div className="accento-input-shell">{parts.input}{parts.action}</div>{parts.description}{parts.error}{parts.counter}</>;
}


export function useFormReset(ref: RefObject<HTMLElement | null>, enabled: boolean, reset: () => void) {
  useEffect(() => {
    if (!enabled) return;
    const element = ref.current;
    const form = element instanceof HTMLInputElement ? element.form : element?.closest('form');
    if (!form) return;
    const handleReset = (event: Event) => queueMicrotask(() => { if (!event.defaultPrevented) reset(); });
    form.addEventListener('reset', handleReset);
    return () => form.removeEventListener('reset', handleReset);
  }, [ref, enabled, reset]);
}
