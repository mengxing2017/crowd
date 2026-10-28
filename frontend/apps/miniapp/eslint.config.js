import js from '@eslint/js'
import ts from 'typescript-eslint'
import vue from 'eslint-plugin-vue'
export default ts.config(
  { ignores: ['dist', 'node_modules'] },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...vue.configs['flat/essential'],
  {
    files: ['src/**/*.{ts,vue}'],
    languageOptions: { globals: { uni: 'readonly', UniApp: 'readonly', console: 'readonly' } },
    rules: {
      'vue/multi-word-component-names': 'off',
      'no-restricted-imports': ['error', { patterns: [{ group: ['@crowd/api-contract', '@crowd/api-contract/*'], message: '只在 feature/api/index.ts 引用接口包。' }] }],
      'no-restricted-globals': ['error', { name: 'fetch', message: '使用 feature/api client' }],
    },
  },
  { files: ['src/features/**/api/index.ts'], rules: { 'no-restricted-imports': ['error', { paths: [{ name: '@crowd/api-contract/community', message: '使用实际所属领域的接口入口。' }] }] } },
  {
    files: ['src/**/*.vue'],
    languageOptions: { parserOptions: { parser: ts.parser, extraFileExtensions: ['.vue'] } },
  },
)
