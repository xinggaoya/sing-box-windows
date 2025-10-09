/**
 * 内核管理性能测试脚本
 * 用于测试重构后的内核服务性能
 */
import { kernelService } from '@/services/kernel-service'

class KernelPerformanceTester {
  private results: Array<{
    operation: string
    duration: number
    success: boolean
    error?: string
  }> = []

  private async measureOperation<T>(
    operationName: string,
    operation: () => Promise<T>
  ): Promise<T> {
    const startTime = performance.now()
    try {
      const result = await operation()
      const duration = performance.now() - startTime
      this.results.push({
        operation: operationName,
        duration,
        success: true
      })
      return result
    } catch (error) {
      const duration = performance.now() - startTime
      this.results.push({
        operation: operationName,
        duration,
        success: false,
        error: error instanceof Error ? error.message : String(error)
      })
      throw error
    }
  }

  async runPerformanceTest(): Promise<void> {
    console.log('🚀 开始内核管理性能测试...')
    console.log('=' .repeat(50))

    // 测试1: 状态查询性能（测试缓存效果）
    console.log('\n📊 测试1: 状态查询性能（测试缓存效果）')
    await this.testStatusQueryPerformance()

    // 测试2: 启动/停止性能
    console.log('\n🔄 测试2: 启动/停止性能')
    await this.testStartStopPerformance()

    // 测试3: 并发操作性能
    console.log('\n⚡ 测试3: 并发操作性能')
    await this.testConcurrentOperations()

    // 测试4: 错误处理性能
    console.log('\n❌ 测试4: 错误处理性能')
    await this.testErrorHandlingPerformance()

    // 输出结果
    this.printResults()
  }

  private async testStatusQueryPerformance(): Promise<void> {
    console.log('连续状态查询测试...')
    
    // 第一次查询（无缓存）
    await this.measureOperation('状态查询-第1次', () => 
      kernelService.getKernelStatus()
    )
    
    // 第二次查询（有缓存）
    await this.measureOperation('状态查询-第2次', () => 
      kernelService.getKernelStatus()
    )
    
    // 第三次查询（有缓存）
    await this.measureOperation('状态查询-第3次', () => 
      kernelService.getKernelStatus()
    )
    
    // 等待缓存过期
    await new Promise(resolve => setTimeout(resolve, 2500))
    
    // 缓存过期后的查询
    await this.measureOperation('状态查询-缓存过期', () => 
      kernelService.getKernelStatus()
    )
  }

  private async testStartStopPerformance(): Promise<void> {
    console.log('内核启动/停止性能测试...')
    
    // 测试启动性能（如果内核已运行，会快速返回）
    await this.measureOperation('内核启动', () => 
      kernelService.startKernel({ timeoutMs: 5000 })
    )
    
    // 等待2秒
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // 测试停止性能
    await this.measureOperation('内核停止', () => 
      kernelService.stopKernel()
    )
  }

  private async testConcurrentOperations(): Promise<void> {
    console.log('并发操作性能测试...')
    
    const concurrentOperations = Array(5).fill(0).map((_, i) => 
      this.measureOperation(`并发状态查询-${i + 1}`, () => 
        kernelService.getKernelStatus()
      )
    )
    
    await Promise.allSettled(concurrentOperations)
  }

  private async testErrorHandlingPerformance(): Promise<void> {
    console.log('错误处理性能测试...')
    
    // 测试无效操作
    await this.measureOperation('无效启动-预期失败', async () => {
      try {
        await kernelService.startKernel({ config: { proxy_mode: 'invalid' as any } })
        return 'unexpected_success'
      } catch (error) {
        console.warn('Expected error caught:', error)
        return 'expected_error'
      }
    })
    
    // 测试健康检查
    await this.measureOperation('健康检查', () => 
      kernelService.checkKernelHealth()
    )
  }

  private printResults(): void {
    console.log('\n📈 性能测试结果')
    console.log('=' .repeat(50))
    
    const operations = new Map<string, Array<{ duration: number; success: boolean }>>()
    
    this.results.forEach(result => {
      if (!operations.has(result.operation)) {
        operations.set(result.operation, [])
      }
      operations.get(result.operation)!.push({
        duration: result.duration,
        success: result.success
      })
    })
    
    operations.forEach((results, operation) => {
      const durations = results.map(r => r.duration)
      const successCount = results.filter(r => r.success).length
      const avgDuration = durations.reduce((sum, d) => sum + d, 0) / durations.length
      const minDuration = Math.min(...durations)
      const maxDuration = Math.max(...durations)
      
      console.log(`\n🔧 ${operation}`)
      console.log(`   成功率: ${successCount}/${results.length} (${(successCount / results.length * 100).toFixed(1)}%)`)
      console.log(`   平均耗时: ${avgDuration.toFixed(2)}ms`)
      console.log(`   最小耗时: ${minDuration.toFixed(2)}ms`)
      console.log(`   最大耗时: ${maxDuration.toFixed(2)}ms`)
      
      if (successCount < results.length) {
        console.log(`   ⚠️  有 ${results.length - successCount} 次失败`)
      }
    })
    
    // 总体统计
    const totalOperations = this.results.length
    const totalSuccess = this.results.filter(r => r.success).length
    const totalDuration = this.results.reduce((sum, r) => sum + r.duration, 0)
    const avgDuration = totalDuration / totalOperations
    
    console.log('\n📊 总体统计')
    console.log('=' .repeat(50))
    console.log(`总操作数: ${totalOperations}`)
    console.log(`成功率: ${totalSuccess}/${totalOperations} (${(totalSuccess / totalOperations * 100).toFixed(1)}%)`)
    console.log(`总耗时: ${totalDuration.toFixed(2)}ms`)
    console.log(`平均耗时: ${avgDuration.toFixed(2)}ms`)
    
    // 性能评估
    console.log('\n🎯 性能评估')
    console.log('=' .repeat(50))
    if (avgDuration < 100) {
      console.log('✅ 性能优秀 - 平均响应时间 < 100ms')
    } else if (avgDuration < 500) {
      console.log('👍 性能良好 - 平均响应时间 < 500ms')
    } else if (avgDuration < 1000) {
      console.log('⚠️  性能一般 - 平均响应时间 < 1s')
    } else {
      console.log('❌ 性能需要优化 - 平均响应时间 > 1s')
    }
    
    if (totalSuccess / totalOperations > 0.95) {
      console.log('✅ 稳定性优秀 - 成功率 > 95%')
    } else if (totalSuccess / totalOperations > 0.9) {
      console.log('👍 稳定性良好 - 成功率 > 90%')
    } else {
      console.log('⚠️  稳定性需要改进 - 成功率 < 90%')
    }
  }

  // 导出结果用于分析
  getResults() {
    return {
      results: this.results,
      summary: {
        totalOperations: this.results.length,
        successCount: this.results.filter(r => r.success).length,
        totalDuration: this.results.reduce((sum, r) => sum + r.duration, 0),
        averageDuration: this.results.reduce((sum, r) => sum + r.duration, 0) / this.results.length
      }
    }
  }
}

// 导出测试器
export const kernelPerformanceTester = new KernelPerformanceTester()

// 提供便捷的测试方法
export async function runKernelPerformanceTest(): Promise<void> {
  try {
    await kernelPerformanceTester.runPerformanceTest()
  } catch (error) {
    console.error('性能测试失败:', error)
    throw error
  }
}

// 如果在开发环境中，自动运行测试
if (import.meta.env.DEV) {
  console.log('🔧 开发模式检测到内核管理重构，性能测试器已就绪')
  console.log('调用 runKernelPerformanceTest() 来运行性能测试')
}