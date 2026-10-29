export function record(value: unknown): value is Record<string, unknown> { return typeof value === 'object' && value !== null && !Array.isArray(value) }

export function safeIntegers(value: unknown): boolean {
 if (typeof value === 'number') return Number.isSafeInteger(value)
 if (Array.isArray(value)) return value.every(safeIntegers)
 if (record(value)) return Object.values(value).every(safeIntegers)
 return true
}
