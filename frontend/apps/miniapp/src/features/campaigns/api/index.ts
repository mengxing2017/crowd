import { createCampaigns } from '@crowd/api-contract/campaigns'
import { transport } from '../../session/api'
export const campaigns = createCampaigns(transport)
export type * from '@crowd/api-contract/campaigns'
export const campaignsCommands = {
 campaign_save: campaigns.campaignSave,
 campaign_transfer: campaigns.campaignTransfer,
 campaign_archive: campaigns.campaignArchive,
}
