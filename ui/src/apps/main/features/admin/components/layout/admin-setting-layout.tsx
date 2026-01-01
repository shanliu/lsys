import { Outlet } from '@tanstack/react-router'

interface AdminSettingLayoutProps {
  className?: string
}

export function AdminSettingLayout({ className }: AdminSettingLayoutProps) {
  return <div >
    <Outlet />
  </div>

}
