import type { PriceRule, TaskTemplate } from '../tasks/types.js'

export interface ProductImage { file_id: string; alt: string }
export interface ProductTask { template_id?: string; price_id?: string; template_preview?: TaskTemplate; price_preview?: PriceRule; id: string; name: string; version: number; completion: string; evidence: string; rule_version: string; commission_cents: number; currency: 'CNY'; rule_starts_at: number; rule_ends_at: number; allow_self: boolean; submit_seconds: number; capacity?: number | null; digital_limit_cents?: number | null }
export interface ProductContent { name: string; category: string; summary: string; details: string; limitations: string; estimated_minutes: number; regions: string[]; qualifications: string[]; images: ProductImage[]; tasks: ProductTask[] }
export interface Category { id: string; version: number; name: string; enabled: boolean; sort: number; reason: string }
export interface ProductRelease { id: string; content: ProductContent; starts_at: number; ends_at: number; status: 'pending' | 'approved' | 'rejected' | 'withdrawn'; submitted_by: string; editor: string; review_reason?: string }
export interface Product { id: string; version: number; draft: Partial<ProductContent>; releases: ProductRelease[]; archived: boolean; offline: boolean }
export interface CatalogAdmin { categories: Category[]; products: Product[]; revision: number; server_time: number }
export interface PublicProduct { id: string; release_id: string; content: ProductContent; starts_at: number; ends_at: number; available_tasks: string[] }
export type PublicCategory = Pick<Category, 'id' | 'version' | 'name' | 'sort' | 'enabled'>
export interface PublicCatalog { categories: PublicCategory[]; products: PublicProduct[]; revision: number }
export interface CatalogNotification { recipient: string; product_id: string; kind: string; time: number; revision: number }
export interface CatalogQuery { keyword?: string; category?: string; min_cents?: number; max_cents?: number; sort?: 'name' | 'commission_asc' | 'commission_desc'; task_status?: 'available' | 'unavailable' }
