import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/')({
  component: RouteComponent,
})

function RouteComponent() {
  const hostname = window.location.hostname;
  const oldhost = `http://${hostname}:8088/`;
  return <div className='m-6'>  <p>
    这是一个服务端以 rust(actix-web)语言, 网页端以reactjs 为基础开发的项目。
  </p>
    <p>
      目标：作为 内部应用管理 及 轻量级开放平台 的解决方案
    </p>
    <p>
      目前为 项目 <b>dev分支</b> 代码,还在<b>迭代开发</b>中，将不定期更新。
    </p>
    <p>
      <b>master 分支</b> 示例请点击: <a className="text-blue-800" href={oldhost} target="_blank">master示例</a>
    </p>
    <p>
      请不要恶意使用本示例项目,如有需要可以 克隆本项目代码 自行部署测试。
    </p>
    <p>
      发现bug,可发送邮件:<a href="mailto:shan.liu@msn.com">shan.liu#msn.com</a>(把#改为@)
    </p>
    <p>
      <b>本项目代码协议: Apache-2.0 license </b>
    </p></div >
}
