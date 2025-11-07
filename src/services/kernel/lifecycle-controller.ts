export type KernelOperation = 'start' | 'stop'

export class KernelLifecycleController {
  private activeOperation: KernelOperation | null = null

  isActive(operation: KernelOperation): boolean {
    return this.activeOperation === operation
  }

  async run<T>(
    operation: KernelOperation,
    onBusy: (activeOperation: KernelOperation) => T,
    handler: () => Promise<T>
  ): Promise<T> {
    if (this.activeOperation) {
      return onBusy(this.activeOperation)
    }

    this.activeOperation = operation
    try {
      return await handler()
    } finally {
      this.activeOperation = null
    }
  }
}
