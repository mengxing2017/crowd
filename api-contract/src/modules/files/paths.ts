export function filePath(id: string, reason?: string): string { return `/api/v1/community/files/${encodeURIComponent(id)}${reason === undefined ? '' : `?reason=${encodeURIComponent(reason)}`}` }
export function uploadPath(purpose: string): string { return `/api/v1/community/files/${encodeURIComponent(purpose)}` }
