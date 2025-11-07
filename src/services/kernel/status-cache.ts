interface CacheRecord<T> {
  value: T
  timestamp: number
}

export class StatusCache<T> {
  private readonly cache = new Map<string, CacheRecord<T>>()

  constructor(private readonly ttlMs: number) {}

  get(key: string): T | undefined {
    const record = this.cache.get(key)
    if (!record) {
      return undefined
    }

    if (Date.now() - record.timestamp > this.ttlMs) {
      this.cache.delete(key)
      return undefined
    }

    return record.value
  }

  set(key: string, value: T): void {
    this.cache.set(key, { value, timestamp: Date.now() })
  }

  clear(): void {
    this.cache.clear()
  }
}
