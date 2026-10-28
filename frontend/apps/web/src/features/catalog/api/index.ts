import { createCatalog } from '@crowd/api-contract/catalog'
import { transport } from '../../session/api'

export const catalog = createCatalog(transport)
export type * from '@crowd/api-contract/catalog'

export const catalogCommands = {
  category_save: catalog.categorySave,
  category_migrate: catalog.categoryMigrate,
  product_save: catalog.productSave,
  product_submit: catalog.productSubmit,
  product_review: catalog.productReview,
  product_offline: catalog.productOffline,
  product_archive: catalog.productArchive,
  product_rollback: catalog.productRollback,
}
