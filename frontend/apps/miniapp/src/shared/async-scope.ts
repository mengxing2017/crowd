// A cleared/hidden view must not accept late asynchronous results.
export function createAsyncScope() {
  let generation = 0
  let active = true
  return {
    capture: () => generation,
    current: (ticket: number) => active && ticket === generation,
    invalidate: () => { generation += 1 },
    hide: () => { active = false; generation += 1 },
    show: () => { active = true },
  }
}
