export interface Receipt { id: string; method?: 'wechat' | 'alipay'; account?: string; name?: string; file_id?: string; status: string; version?: number; disables?: string }
