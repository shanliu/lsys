import { createFileRoute } from '@tanstack/react-router'

function AdminIndexPage() {
  return (
    <div className="container mx-auto p-6">
      <h1 className="text-2xl font-bold mb-6">系统管理</h1>
      <p>欢迎使用系统管理后台</p>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-6">
        <div className="p-4 border rounded-lg">
          <h3 className="font-semibold">应用管理</h3>
          <p className="text-sm text-gray-600">管理系统中的应用</p>
        </div>
        <div className="p-4 border rounded-lg">
          <h3 className="font-semibold">用户管理</h3>
          <p className="text-sm text-gray-600">管理系统用户</p>
        </div>
        <div className="p-4 border rounded-lg">
          <h3 className="font-semibold">系统配置</h3>
          <p className="text-sm text-gray-600">系统参数配置</p>
        </div>
      </div>
    </div>
  )
}

export const Route = createFileRoute('/_main/admin/')({
  component: AdminIndexPage,
})
