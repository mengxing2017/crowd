export type Run = (work: () => Promise<void>) => Promise<boolean>
export type Save = (action: string, body: Record<string, unknown>) => Promise<boolean>
