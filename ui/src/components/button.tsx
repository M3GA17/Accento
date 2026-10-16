'use client';
import React, { forwardRef } from 'react';
import { Button as AriaButton, type ButtonProps as AriaButtonProps } from 'react-aria-components';

export type ButtonProps = Omit<AriaButtonProps, 'children' | 'isDisabled'> & {
  variant?: 'primary' | 'secondary' | 'quiet' | 'danger';
  children?: React.ReactNode;
  disabled?: boolean;
  loading?: boolean;
  loadingLabel?: string;
};
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = 'secondary', className = '', disabled = false, loading = false, loadingLabel, children, type = 'button', render: renderElement, ...props }, ref,
) {
  return <AriaButton {...props} className={'accento-button ' + className} data-variant={variant} ref={ref} type={type} isDisabled={disabled || loading}
    data-loading={loading || undefined}
    render={(domProps, state) => {
      const accessibleProps = {...domProps, 'aria-busy': loading || undefined,
        style: {minWidth:'var(--accento-target-width, 24px)',minHeight:'var(--accento-target-height, 24px)',...domProps.style}};
      return renderElement ? renderElement(accessibleProps, state) : <button {...accessibleProps}/>;
    }}>
    {loading && loadingLabel ? loadingLabel : children}
  </AriaButton>;
});
