/**
 * Bundle 分析工具
 * 用于分析和优化打包文件大小
 */

// 模块加载统计
interface ModuleStats {
  name: string
  size: number
  loadTime: number
  loadCount: number
  dependencies: string[]
}

// Bundle 分析器
export class BundleAnalyzer {
  private moduleStats = new Map<string, ModuleStats>()
  private chunkMap = new Map<string, string[]>()
  private startTime = performance.now()

  /**
   * 记录模块加载
   */
  recordModuleLoad(moduleName: string, size: number, dependencies: string[] = []) {
    const existing = this.moduleStats.get(moduleName)
    const loadTime = performance.now()

    if (existing) {
      existing.loadCount++
      existing.loadTime = loadTime
    } else {
      this.moduleStats.set(moduleName, {
        name: moduleName,
        size,
        loadTime: loadTime - this.startTime,
        loadCount: 1,
        dependencies,
      })
    }
  }

  /**
   * 记录代码分割chunk
   */
  recordChunk(chunkName: string, modules: string[]) {
    this.chunkMap.set(chunkName, modules)
  }

  /**
   * 获取最大的模块
   */
  getLargestModules(count = 10): ModuleStats[] {
    return Array.from(this.moduleStats.values())
      .sort((a, b) => b.size - a.size)
      .slice(0, count)
  }

  /**
   * 获取最慢加载的模块
   */
  getSlowestModules(count = 10): ModuleStats[] {
    return Array.from(this.moduleStats.values())
      .sort((a, b) => b.loadTime - a.loadTime)
      .slice(0, count)
  }

  /**
   * 获取重复加载的模块
   */
  getDuplicateModules(): ModuleStats[] {
    return Array.from(this.moduleStats.values()).filter((module) => module.loadCount > 1)
  }

  /**
   * 分析模块依赖关系
   */
  analyzeDependencies() {
    const dependencyGraph = new Map<string, Set<string>>()
    const reverseDependencies = new Map<string, Set<string>>()

    // 构建依赖图
    for (const [moduleName, stats] of this.moduleStats) {
      dependencyGraph.set(moduleName, new Set(stats.dependencies))

      // 构建反向依赖
      for (const dep of stats.dependencies) {
        if (!reverseDependencies.has(dep)) {
          reverseDependencies.set(dep, new Set())
        }
        reverseDependencies.get(dep)!.add(moduleName)
      }
    }

    return {
      dependencyGraph,
      reverseDependencies,
    }
  }

  /**
   * 检测循环依赖
   */
  detectCircularDependencies(): string[][] {
    const { dependencyGraph } = this.analyzeDependencies()
    const visited = new Set<string>()
    const visiting = new Set<string>()
    const circularDeps: string[][] = []

    function dfs(node: string, path: string[]): void {
      if (visiting.has(node)) {
        // 找到循环依赖
        const cycleStart = path.indexOf(node)
        if (cycleStart !== -1) {
          circularDeps.push(path.slice(cycleStart).concat(node))
        }
        return
      }

      if (visited.has(node)) return

      visiting.add(node)
      path.push(node)

      const deps = dependencyGraph.get(node) || new Set()
      for (const dep of deps) {
        dfs(dep, [...path])
      }

      visiting.delete(node)
      visited.add(node)
      path.pop()
    }

    for (const moduleName of dependencyGraph.keys()) {
      if (!visited.has(moduleName)) {
        dfs(moduleName, [])
      }
    }

    return circularDeps
  }

  /**
   * 生成优化建议
   */
  generateOptimizationSuggestions() {
    const suggestions: string[] = []
    const largestModules = this.getLargestModules(5)
    const slowestModules = this.getSlowestModules(5)
    const duplicateModules = this.getDuplicateModules()
    const circularDeps = this.detectCircularDependencies()

    // 大文件建议
    if (largestModules.length > 0) {
      suggestions.push(
        `发现 ${largestModules.length} 个大模块:`,
        ...largestModules.map((m) => `  - ${m.name}: ${(m.size / 1024).toFixed(2)}KB`),
      )
      suggestions.push('建议: 考虑代码分割或懒加载这些大模块')
    }

    // 慢加载建议
    if (slowestModules.length > 0) {
      suggestions.push(
        `发现 ${slowestModules.length} 个慢加载模块:`,
        ...slowestModules.map((m) => `  - ${m.name}: ${m.loadTime.toFixed(2)}ms`),
      )
      suggestions.push('建议: 优化这些模块的加载时机或使用预加载')
    }

    // 重复加载建议
    if (duplicateModules.length > 0) {
      suggestions.push(
        `发现 ${duplicateModules.length} 个重复加载的模块:`,
        ...duplicateModules.map((m) => `  - ${m.name}: 加载 ${m.loadCount} 次`),
      )
      suggestions.push('建议: 考虑提取为公共模块或使用模块缓存')
    }

    // 循环依赖建议
    if (circularDeps.length > 0) {
      suggestions.push(
        `发现 ${circularDeps.length} 个循环依赖:`,
        ...circularDeps.map((cycle) => `  - ${cycle.join(' → ')}`),
      )
      suggestions.push('建议: 重构代码以消除循环依赖')
    }

    return suggestions
  }

  /**
   * 生成详细报告
   */
  generateReport() {
    const totalModules = this.moduleStats.size
    const totalSize = Array.from(this.moduleStats.values()).reduce(
      (sum, module) => sum + module.size,
      0,
    )
    const averageSize = totalSize / totalModules

    return {
      summary: {
        totalModules,
        totalSize: Math.round(totalSize / 1024), // KB
        averageSize: Math.round(averageSize / 1024), // KB
        chunks: this.chunkMap.size,
      },
      largestModules: this.getLargestModules(),
      slowestModules: this.getSlowestModules(),
      duplicateModules: this.getDuplicateModules(),
      circularDependencies: this.detectCircularDependencies(),
      optimizationSuggestions: this.generateOptimizationSuggestions(),
    }
  }

  /**
   * 输出报告到控制台
   */
  printReport() {
    const report = this.generateReport()

    console.group('📦 Bundle 分析报告')

    console.group('📊 总览')
    console.log(`模块数量: ${report.summary.totalModules}`)
    console.log(`总大小: ${report.summary.totalSize}KB`)
    console.log(`平均大小: ${report.summary.averageSize}KB`)
    console.log(`代码块数量: ${report.summary.chunks}`)
    console.groupEnd()

    if (report.largestModules.length > 0) {
      console.group('🗂️ 最大模块')
      report.largestModules.forEach((module) => {
        console.log(`${module.name}: ${(module.size / 1024).toFixed(2)}KB`)
      })
      console.groupEnd()
    }

    if (report.slowestModules.length > 0) {
      console.group('⏱️ 最慢模块')
      report.slowestModules.forEach((module) => {
        console.log(`${module.name}: ${module.loadTime.toFixed(2)}ms`)
      })
      console.groupEnd()
    }

    if (report.duplicateModules.length > 0) {
      console.group('🔄 重复加载模块')
      report.duplicateModules.forEach((module) => {
        console.log(`${module.name}: ${module.loadCount} 次`)
      })
      console.groupEnd()
    }

    if (report.circularDependencies.length > 0) {
      console.group('🔄 循环依赖')
      report.circularDependencies.forEach((cycle) => {
        console.log(cycle.join(' → '))
      })
      console.groupEnd()
    }

    if (report.optimizationSuggestions.length > 0) {
      console.group('💡 优化建议')
      report.optimizationSuggestions.forEach((suggestion) => {
        console.log(suggestion)
      })
      console.groupEnd()
    }

    console.groupEnd()
  }
}

// 创建全局实例
export const bundleAnalyzer = new BundleAnalyzer()

// 自动记录Vue组件加载
if (import.meta.env.DEV) {
  // 拦截动态import
  const windowWithVite = window as Window & {
    __viteOriginalImport?: (specifier: string) => Promise<unknown>
  }
  const originalImport = windowWithVite.__viteOriginalImport || (() => {})
  if (typeof originalImport === 'function') {
    windowWithVite.__viteOriginalImport = async function (specifier: string) {
      const startTime = performance.now()
      try {
        const module = await originalImport(specifier)
        const endTime = performance.now()
        const loadTime = endTime - startTime

        // 估算模块大小（粗略）
        const estimatedSize = JSON.stringify(module).length

        bundleAnalyzer.recordModuleLoad(specifier, estimatedSize)
        return module
      } catch (error) {
        console.error(`模块加载失败: ${specifier}`, error)
        throw error
      }
    }
  }

  // 定期输出报告（仅开发环境）
  setInterval(() => {
    bundleAnalyzer.printReport()
  }, 60000) // 每分钟输出一次
}
