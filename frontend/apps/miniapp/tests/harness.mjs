import ts from 'typescript'
import { readFileSync, existsSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import vm from 'node:vm'
import { createRequire } from 'node:module'
const localRequire = createRequire(import.meta.url)
export const root = resolve(import.meta.dirname, '../src')
export function harness(overrides = {}, uniOverrides = {}) {
  const hooks = { hide: [], show: [], unmount: [], mounted: [] }
  const storage = new Map(), removed = []
  const uni = {
    getStorageSync: key => storage.get(key), setStorageSync: (key, value) => storage.set(key, structuredClone(value)),
    getFileSystemManager: () => ({ unlink: ({ filePath }) => removed.push(filePath) }),
    ...uniOverrides,
  }
  const vue = { ...localRequire('vue'), onMounted: f => hooks.mounted.push(f), onUnmounted: f => hooks.unmount.push(f) }
  const modules = new Map()
  function load(file) {
    file = resolve(file)
    if (overrides[file]) return overrides[file]
    if (modules.has(file)) return modules.get(file).exports
    const module = { exports: {} }; modules.set(file, module)
    let source = readFileSync(file, 'utf8')
    if (file.endsWith('.vue')) source = source.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1]
    source = source.replaceAll('import.meta.env', '({VITE_API_ORIGIN:"https://api.example.test", VITE_PROMOTION_ORIGIN:"https://go.example.test"})')
    const exports = overrides.expose?.[file]
    if (exports) source += `\nexport { ${exports.join(',')} }`
    const code = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText
    const require = name => {
      if (name === 'vue') return vue
      if (name === '@dcloudio/uni-app') return { onHide: f => hooks.hide.push(f), onShow: f => hooks.show.push(f), onLoad: () => undefined }
      let next
      if (name.startsWith('@crowd/api-contract/')) next = resolve(root, '../../../../api-contract/src/modules', name.split('/').at(-1), 'index.ts')
      else if (name.startsWith('.')) {
        next = resolve(dirname(file), name).replace(/\.js$/, '')
        if (next.endsWith('.vue')) return {}
        if (existsSync(next + '.ts')) next += '.ts'
        else if (existsSync(next + '/index.ts')) next += '/index.ts'
      } else return localRequire(name)
      return load(next)
    }
    vm.runInNewContext(code, { module, exports: module.exports, require, uni, console, setTimeout, clearTimeout, URL, Date, Math, Promise, Set, Map, Error, defineProps: () => overrides.props || {} }, { filename: file })
    return module.exports
  }
  return { load: name => load(resolve(root, name)), hooks, storage, removed, uni }
}
