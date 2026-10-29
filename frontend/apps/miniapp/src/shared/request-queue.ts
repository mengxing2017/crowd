// Leave capacity for native/platform traffic; this is an engineering limit, not a business rule.
export function createRequestQueue(limit = 4) {
  let active = 0
  const waiting: (() => void)[] = []
  return async function schedule<T>(work: () => Promise<T>): Promise<T> {
    if (active >= limit) await new Promise<void>(resolve => waiting.push(resolve))
    else active += 1
    try { return await work() }
    finally {
      const next = waiting.shift()
      if (next) next()
      else active -= 1
    }
  }
}
