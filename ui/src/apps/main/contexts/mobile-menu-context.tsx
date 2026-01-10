import { createContext, useContext } from 'react'

export interface MobileMenuContextType {
  isOpen: boolean
  setOpen: (open: boolean) => void
}

export const MobileMenuContext = createContext<MobileMenuContextType | null>(null)

export function useMobileMenu() {
  const context = useContext(MobileMenuContext)
  if (!context) {
    throw new Error('useMobileMenu must be used within a MobileMenuContext.Provider')
  }
  return context
}
