
import { useIsMobile } from '@shared/hooks/use-mobile';
import { cn } from '@shared/lib/utils';
import copy from 'copy-to-clipboard';
import { Copy } from 'lucide-react';
import { createContext, useCallback, useContext } from 'react';
import { Toaster as Sonner, toast, ToasterProps } from 'sonner';
import { useTheme } from './theme-context';

interface Props {
  children: React.ReactNode
}

interface ToastOptions {
  duration?: number
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' | 'top-center' | 'bottom-center'
}

type ToastState = {
  success: (message: string, options?: ToastOptions) => void
  error: (message: string, options?: ToastOptions) => void
  warning: (message: string, options?: ToastOptions) => void
  info: (message: string, options?: ToastOptions) => void
  loading: (message: string, options?: ToastOptions) => void
  dismiss: (toastId?: string) => void
}

const ToastContext = createContext<ToastState | undefined>(undefined);



export const ToastProvider = ({
  children,
  ...props
}: Props) => {
  const { theme = 'system' } = useTheme();
  const isMobile = useIsMobile();
  const defaultPosition = isMobile ? 'bottom-center' : 'top-right';

  const success = useCallback((message: string, options?: ToastOptions) => {
    toast.success(message, { position: defaultPosition, ...options });
  }, [defaultPosition]);

  const error = useCallback((message: string, options?: ToastOptions) => {
    toast.error(message, {
      position: defaultPosition,
      ...options,
      action: {
        label: <Copy className="h-3 w-3" />,
        onClick: () => {
          copy(message);
        }
      },
      actionButtonStyle: {
        padding: '3px',
        margin: 0,
        marginLeft: 'auto',
        width: '18px',
        height: '18px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      },
      classNames: {
        actionButton: '!bg-foreground/20 hover:!bg-foreground/40 transition-colors'
      }
    });
  }, [defaultPosition]);

  const warning = useCallback((message: string, options?: ToastOptions) => {
    toast(message, {
      position: defaultPosition,
      ...options,
      className: 'bg-warning text-warning-foreground'
    });
  }, [defaultPosition]);

  const info = useCallback((message: string, options?: ToastOptions) => {
    toast(message, {
      position: defaultPosition,
      ...options,
      className: 'bg-info text-info-foreground'
    });
  }, [defaultPosition]);

  const loading = useCallback((message: string, options?: ToastOptions) => {
    toast.loading(message, { position: defaultPosition, ...options });
  }, [defaultPosition]);

  const dismiss = useCallback((toastId?: string) => {
    toast.dismiss(toastId);
  }, []);

  return (
    <ToastContext.Provider
      value={{
        success,
        error,
        warning,
        info,
        loading,
        dismiss
      }}
    >
      <Sonner
        theme={theme as ToasterProps['theme']}
        className={cn("toaster group [&>*]:bg-popover [&>*]:text-popover-foreground [&>*]:border-border [&>.success]:bg-success [&>.success]:text-success-foreground [&>.error]:bg-destructive [&>.error]:text-destructive-foreground")}
        position={defaultPosition}
        toastOptions={{
          classNames: {
            toast: isMobile ? '' : 'mt-[40px]'
          }
        }}
        {...props}
      />
      {children}
    </ToastContext.Provider>
  )
}



// eslint-disable-next-line react-refresh/only-export-components
export function useToast() {
  const context = useContext(ToastContext);

  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
}
