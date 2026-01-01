export default [
  {
    // 全局忽略配置
    ignores: ['**/shared/components/ui/**', '**/routeTree.gen.ts'],
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: {
      parser: (await import('@typescript-eslint/parser')).default,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: {
          jsx: true
        }
      }
    },
    plugins: {
      '@typescript-eslint': (await import('@typescript-eslint/eslint-plugin')).default,
      'unused-imports': (await import('eslint-plugin-unused-imports')).default,
      'react-hooks': (await import('eslint-plugin-react-hooks')).default,
      'react-refresh': (await import('eslint-plugin-react-refresh')).default,
      'import': (await import('eslint-plugin-import')).default
    },
    settings: {
      'import/resolver': {
        alias: {
          map: [
            ['@', './src'],
            ['@shared', './src/shared'],
            ['@apps', './src/apps']
          ],
          extensions: ['.ts', '.tsx', '.js', '.jsx']
        }
      }
    },
    rules: {
      // 未使用的引入 - 不报错，仅在 fix 时清理
      'unused-imports/no-unused-imports': 'off',

      // 关闭 unused-imports 插件的 no-unused-vars 规则
      'unused-imports/no-unused-vars': 'off',

      // 使用 TypeScript ESLint 的 no-unused-vars 规则
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          vars: 'all',
          varsIgnorePattern: '^_',
          args: 'none', // 不检查函数参数，这样接口中的回调参数就不会报错
          argsIgnorePattern: '^_',
          ignoreRestSiblings: true,
          caughtErrors: 'none'
        }
      ],

      // React hooks
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn',
      // React refresh
      'react-refresh/only-export-components': [
        'warn',
        { allowConstantExport: true }
      ],

      // 禁止跨应用导入
      'import/no-restricted-paths': [
        'error',
        {
          zones: [
            // home 应用不能导入 main 应用
            {
              target: './src/apps/home',
              from: './src/apps/main',
              message: 'home 应用不能导入 main 应用的代码，请使用 @shared'
            },
            // main 应用不能导入 home 应用
            {
              target: './src/apps/main',
              from: './src/apps/home',
              message: 'main 应用不能导入 home 应用的代码，请使用 @shared'
            },
            // shared 不能导入任何 apps
            {
              target: './src/shared',
              from: './src/apps',
              message: 'shared 不能导入 apps 的代码'
            }
          ]
        }
      ]
    }
  }
];
