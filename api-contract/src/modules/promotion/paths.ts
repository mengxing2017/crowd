export function qrPath(code: string): string { return `/api/v1/community/qr/${encodeURIComponent(code)}` }
