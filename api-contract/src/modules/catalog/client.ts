import type { Transport } from '../common/types.js'
import type { CatalogQuery, PublicCatalog, CatalogAdmin, CatalogNotification } from './types.js'
import { createRequester } from '../common/request.js'
import { createResource } from '../common/resource.js'
import { validateCatalog } from './validation.js'

export function createCatalog(transport: Transport) {
  const call = createRequester(transport, validateCatalog)
  const resource = createResource(transport, validateCatalog)
  return {
    catalog: (query: CatalogQuery = {}) => {
      const params = Object.entries(query).filter(([, value]) => value !== undefined && value !== '').map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`).join('&')
      return call<PublicCatalog>({ method: 'GET', path: `/api/v1/catalog${params ? `?${params}` : ''}` })
    },
    admin: () => resource.read<CatalogAdmin>('catalog_admin'),
    notifications: () => resource.read<CatalogNotification[]>('catalog_notifications'),
    categorySave: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('category_save', body, key),
    categoryMigrate: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('category_migrate', body, key),
    productSave: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('product_save', body, key),
    productSubmit: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('product_submit', body, key),
    productReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('product_review', body, key),
    productOffline: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('product_offline', body, key),
    productArchive: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('product_archive', body, key),
    productRollback: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('product_rollback', body, key),
  }
}
