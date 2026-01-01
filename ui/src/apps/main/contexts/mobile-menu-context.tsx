import { createContext, useContext, useState, type ReactNode } from 'react'

interface MobileMenuContextType {
  isOpen: boolean
  setOpen: (open: boolean) => void
}

const MobileMenuContext = createContext<MobileMenuContextType | null>(null)

export function MobileMenuProvider({ children }: { children: ReactNode }) {
  const [isOpen, setIsOpen] = useState(false)

  return (
    <MobileMenuContext.Provider value={{ isOpen, setOpen: setIsOpen }}>
      {children}
    </MobileMenuContext.Provider>
  )
}

export function useMobileMenu() {
  const context = useContext(MobileMenuContext)
  if (!context) {
    throw new Error('useMobileMenu must be used within a MobileMenuProvider')
  }
  return context
}
