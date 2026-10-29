import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import { validate } from './validation.js'
import type { CampaignsRecord } from './types.js'
export function createCampaigns(transport: Transport) {
 const resource = createResource(transport, validate)
 return {
    campaigns: (target?: string) => resource.read<CampaignsRecord[]>('campaigns', target),
    campaignReport: (target?: string) => resource.read<Record<string, unknown>>('campaign_report', target),
    campaignSave: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('campaign_save', body, key),
    campaignTransfer: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('campaign_transfer', body, key),
    campaignArchive: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('campaign_archive', body, key),
 }
}
