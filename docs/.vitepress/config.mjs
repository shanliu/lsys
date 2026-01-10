import { defineConfig } from 'vitepress'
import sidebarRest from './sidebar_rest.json' assert { type: 'json' };
import sidebarApi from './sidebar_api.json' assert { type: 'json' };
export default defineConfig({
  title: "lsys 在线文档",
  base: '/docs/',
  outDir: '../ui/dist/docs',
  themeConfig: {
    nav: [
       {
        text: '文档',
        items: [
          {
            text: '应用接口文档',
            link: '/rest/',
          },
          {
            text: '系统接口文档',
            link: '/api/',
          }
        ],
      },
      {
        text: '示例',
        items: [
          {
            text: '在线示例',
            link: 'https://lsys.cc',
          },
          {
            text: 'SDK及示例',
            link: 'http://lsys.cc:8081/',
          }
        ],
      }
    ],

    sidebar:{
      '/rest/': sidebarRest,
      '/api/':sidebarApi,
    },
    editLink: {
      pattern: 'https://github.com/shanliu/lsys/tree/dev/docs/:path'
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/shanliu/lsys' }
    ],

    outline: {
      label: '概要',
      level: 'deep'
    }
  }
})
