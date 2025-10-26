export default {
  // 通用词汇
  common: {
    appName: 'Sing-Box Windows',
    ok: '确定',
    cancel: '取消',
    confirm: '确认',
    save: '保存',
    delete: '删除',
    edit: '编辑',
    error: '错误',
    success: '成功',
    refresh: '刷新',
    unknown: '未知',
    restart: '重启',
    search: '搜索',
    later: '稍后',
    minimizedToTray: '已最小化到托盘',
  },

  // 导航菜单
  nav: {
    navigation: '导航',
    home: '首页',
    proxy: '代理',
    connections: '连接',
    logs: '日志',
    rules: '规则',
    subscription: '订阅',
    settings: '设置',
    expand: '展开',
    collapse: '收起',
  },

  // 状态相关
  status: {
    running: '运行中',
    stopped: '已停止',
    starting: '启动中',
    stopping: '停止中',
    restarting: '重启中',
    connecting: '连接中',
    disconnected: '已断开',
  },

  // 首页
  home: {
    // 基础操作
    start: '启动',
    stop: '停止',
    restart: '重启',
    restartAsAdmin: '以管理员重启',

    // 状态信息
    startSuccess: '启动成功',
    startFailed: '启动失败',
    stopSuccess: '停止成功',
    stopFailed: '停止失败',
    restartFailed: '重启失败',
    nodeModeChangeSuccess: '节点模式切换成功',
    nodeModeChangeFailed: '节点模式切换失败',

    // WebSocket状态
    wsStatus: {
      connected: '已连接',
      disconnected: '未连接',
    },

    // 管理员状态
    adminStatus: {
      admin: '管理员',
      normal: '普通用户',
    },

    // 代理头部
    proxyHeader: {
      flowMode: '流量代理模式',
      nodeMode: '节点代理模式',
    },

    // 状态描述
    status: {
      startingDesc: '正在启动内核，请稍候...',
      stoppingDesc: '正在停止内核，请稍候...',
      runningDesc: '内核正在运行，代理服务可用',
      stoppedDesc: '内核已停止，代理服务不可用',
      disconnectedDesc: '内核已启动但连接异常，请检查配置',
    },

    // 流量统计
    traffic: {
      title: '流量统计',
      uploadSpeed: '上传速度',
      downloadSpeed: '下载速度',
      uploadTotal: '上传总流量',
      downloadTotal: '下载总流量',
      memory: '内存占用',
      connectionsLabel: '个连接',
    },

    // 流量代理模式
    proxyMode: {
      system: '系统代理',
      tun: 'TUN模式',
      manual: '手动模式',
      systemTip: '自动设置系统代理',
      tunTip: '使用TUN虚拟网卡代理所有流量，需要管理员权限',
      manualTip: '需要手动配置系统代理',
      systemDescription: '自动配置系统代理设置，为浏览器和应用程序提供代理服务',
      tunDescription: '通过虚拟网卡捕获并代理所有系统流量，需要管理员权限',
      manualDescription: '需要在应用程序和浏览器中手动配置代理设置',
    },

    // 节点代理模式
    nodeMode: {
      global: '全局模式',
      rule: '规则模式',
      globalTip: '全部流量走代理',
      ruleTip: '根据规则智能分流',
    },
  },

  // 代理页面
  proxy: {
    title: '代理设置',
    subtitle: '代理节点管理与延迟测试',
    search: '搜索',
    nodes: '节点',
    nodeCount: '个节点',
    testNode: '测试',
    testing: '测试中...',
    timeout: '超时',
    testProgress: '测试进度:',
    switch: '切换',
    switchFailed: '切换失败',
    switchErrorMessage: '切换失败，请检查Sing-Box是否已启动',
    testErrorMessage: '延迟测试失败，可能是节点无法连接或API未响应',
    testFailed: '延迟测试失败',
    nodeTestFailed: '节点延迟测试失败',
    nodeTestComplete: '节点延迟测试完成',
    groupTestComplete: '组延迟测试完成',
    batchTestComplete: '批量延迟测试完成',
    loadSuccess: '代理列表加载成功',
    loadFailed: '获取代理列表失败',
    loadFailedCheck: '获取代理列表失败，请检查Sing-Box是否已启动',
    loadingInfo: '正在加载代理信息...',
    noProxyGroups: '暂无代理组',
    checkConfigOrRefresh: '请检查配置文件或刷新页面',
    currentLabel: '当前节点:',
  },

  // 连接页面
  connections: {
    title: '连接管理',
    subtitle: '管理和监控网络连接',
    source: '来源',
    destination: '目标',
    networkType: '网络类型',
    rule: '规则',
    traffic: '流量',
    activeConnections: '活跃连接',
    refreshConnections: '刷新连接',
    refreshSuccess: '连接列表已刷新',
    refreshError: '刷新连接列表失败',
    noActiveConnections: '当前没有活跃的网络连接',
    adjustSearchOrFilters: '尝试调整搜索条件或筛选器',
    searchPlaceholder: '搜索连接 (ID、IP、域名、进程)',
    networkTypeFilter: '网络类型',
    ruleFilter: '规则筛选',
    noMatchingConnections2: '无匹配连接',
    matchedConnections: '匹配连接',
  },

  // 日志页面
  log: {
    title: '日志查看',
    subtitle: '实时监控系统日志',
    search: '搜索',
    searchLogs: '搜索日志...',
    clear: '清空日志',
    copy: '复制日志',
    export: '导出日志',
    records: '条记录',
    noLogs: '暂无日志记录',
    noLogsDesc: '当前没有日志记录',
    clearedSuccess: '日志已清空',
    copiedSuccess: '日志已复制到剪贴板',
    copyFailed: '复制失败',
    exportedSuccess: '日志已导出',
    clearSearch: '清除搜索',
    filterType: '筛选日志类型',
    autoScroll: '自动滚动',
    manualScroll: '手动滚动',
    types: {
      info: '信息',
      warning: '警告',
      error: '错误',
      success: '成功',
    },
  },

  // 规则页面
  rules: {
    title: '规则管理',
    subtitle: '管理代理路由规则',
    search: '搜索',
    add: '添加规则',
    type: '规则类型',
    content: '规则内容',
    targetProxy: '目标代理',
    directConnect: '本地直连',
    blockAction: '拦截',
    noRulesData: '暂无规则数据',
    adjustSearchOrFilters: '请尝试调整搜索条件或筛选条件',
    clickRefreshToGetRules: '点击刷新按钮获取规则数据',
    getRules: '获取规则',
    clearFilters: '清除筛选',
    searchPlaceholder: '搜索规则内容、类型、代理...',
    ruleTypes: '规则类型',
    proxyTargets: '代理目标',
    totalRules: '总规则',
    matchingRules: '匹配规则',
  },

  // 订阅页面
  sub: {
    title: '订阅管理',
    subtitle: '管理您的订阅配置和代理节点',
    total: '总计',
    active: '活跃',
    add: '添加订阅',
    edit: '编辑订阅',
    editConfig: '编辑配置',
    editCurrentConfig: '编辑当前配置',
    saveAndApply: '保存并应用',
    inUse: '使用中',
    manual: '手动编辑',
    deleteSuccess: '删除成功',
    cannotDeleteActive: '无法删除正在使用的订阅',
    name: '名称',
    url: '链接',
    content: '内容',
    namePlaceholder: '请输入订阅名称',
    urlPlaceholder: '请输入订阅链接',
    manualContentPlaceholder: '请输入配置内容（JSON格式）',
    configContentPlaceholder: '配置文件内容（JSON格式）',
    nameRequired: '请输入订阅名称',
    urlRequired: '请输入订阅链接',
    contentRequired: '请输入配置内容',
    invalidUrl: '请输入有效的URL',
    neverUsed: '从未使用',
    addAndUseSuccess: '添加并应用订阅成功',
    updateSuccess: '更新订阅成功',
    operationFailed: '操作失败: ',
    use: '使用',
    useSuccess: '应用成功',
    useFailed: '应用失败: ',
    copyLink: '复制链接',
    linkCopied: '链接已复制到剪贴板',
    lastUpdate: '更新时间',
    noSubs: '暂无订阅',
    noSubscriptionsYet: '您还没有添加任何订阅',
    addFirstSubscription: '添加第一个订阅',
    readConfigFailed: '读取配置失败: ',
    configSaved: '配置已保存并应用',
    saveConfigFailed: '保存配置失败: ',
    useOriginal: '使用原始订阅',
    useOriginalConfig: '直接使用原始订阅（仅替换端口）',
    useExtractedNodes: '提取节点到本地模板',
    urlSubscription: 'URL订阅',
    manualConfig: '手动配置',
    manualContent: '手动配置内容',
  },

  // 设置页面
  setting: {
    title: '设置',
    subtitle: '管理应用程序配置和系统设置',
    appVersion: '应用版本',
    kernelVersion: '内核版本',
    notInstalled: '未安装',
    newVersionFound: '发现新版本',

    // 通用设置
    general: {
      title: '常规设置',
      description: '配置语言、网络和其他应用程序选项',
    },

    // 主题设置
    theme: {
      light: '浅色',
      dark: '深色',
    },

    // 语言设置
    language: {
      title: '语言',
      auto: '自动',
      changed: '语言已更改',
      changeSuccess: '语言已成功更改',
      description: '选择应用程序界面显示语言',
    },

    // 网络设置
    network: {
      ipv6: 'IPv6 支持',
      ipv4Only: '仅使用 IPv4',
      ipv6Desc: '启用或禁用 IPv6 网络协议支持',
      ipVersionChanged: 'IP 版本设置已更改',
      ports: '端口设置',
      portsDesc: '配置代理和API端口',
      configure: '配置端口',
      proxyPort: '代理端口',
      apiPort: 'API 端口',
      portSettings: '端口设置',
      invalidPort: '无效的端口号，请输入 1024-65535 之间的端口',
      portConflict: '代理端口和 API 端口不能相同',
      restartRequired: '需要重启内核',
      restartDesc: '端口更改后需要重启内核才能生效',
      portChanged: '端口设置已更改',
      portChangeSuccess: '端口配置已成功更新',
    },

    // 更新设置
    update: {
      title: '更新设置',
      description: '配置应用自动更新和版本检查选项',
      check: '检查更新',
      autoCheck: '自动检查更新',
      autoCheckDesc: '启动时自动检查应用更新',
      acceptPrerelease: '接收测试版本',
      acceptPrereleaseDesc: '允许接收和安装预发布版本（测试版本）',
      current: '当前版本',
      newVersion: '新版本',
      confirmUpdate: '是否立即更新？',
      later: '下次再说',
      updateNow: '立即更新',
      downloading: '正在下载更新',
      installing: '正在安装',
      preparingDownload: '准备下载...',
      checkError: '检查更新失败',
      updateFailed: '更新失败',
      alreadyLatest: '当前已是最新版本',
      beta: '测试版',
      releaseNotes: '更新日志',
      updateNotice: '更新提醒',
      prereleaseWarningDesc: '测试版本可能包含未稳定的功能和潜在的问题，建议仅在测试环境中使用。',
      prereleaseConfirm: '确认启用测试版本',
      prereleaseConfirmDesc: '启用后，您将接收到预发布版本的更新通知。这些版本可能不够稳定，确定要继续吗？',
      prereleaseEnabled: '测试版本已启用',
      prereleaseEnabledDesc: '您将接收到预发布版本的更新通知',
      prereleaseDisabled: '测试版本已禁用',
      prereleaseDisabledDesc: '您将只接收正式版本的更新通知',
    },

    // 自启动设置
    autoStart: {
      app: '开机自启',
      appDesc: '系统启动时自动运行应用程序',
      kernel: '启动内核',
      kernelDesc: '应用启动时自动运行内核服务',
    },

    // 内核自动启动
    kernelAutoStart: {
      enabled: '自动启动内核已启用',
      disabled: '自动启动内核已禁用',
      enableSuccess: '应用启动时将自动启动内核',
      disableSuccess: '应用启动时不会自动启动内核',
    },

    // 启动设置
    startup: {
      title: '启动设置',
      description: '配置应用启动和自动运行选项',
      enabled: '开机自启动已启用',
      disabled: '开机自启动已禁用',
      enableSuccess: '开机自启动设置成功',
      disableSuccess: '开机自启动已禁用',
    },

    // 内核管理
    kernel: {
      title: '内核管理',
      description: '管理 Sing-Box 内核版本和下载',
      download: '下载内核',
      checkInstall: '检查安装',
      downloadComplete: '内核下载完成！',
      installSuccess: '内核安装验证成功！',
      installFailed: '未检测到有效的内核文件',
      downloadFailed: '下载失败',
      preparingDownload: '准备下载...',
      downloading: '正在下载内核',
      downloadingDescription: '内核文件正在下载中，请稍候...',
      manualDownload: '手动下载',
      manualDownloadTitle: '手动下载说明',
      manualDownloadGuide: '请按照以下步骤操作：\n1. 访问 https://github.com/SagerNet/sing-box/releases/latest\n2. 下载对应系统版本的 sing-box\n3. 将解压后的可执行文件放置在以下目录：\nWindows: 用户目录/AppData/Local/sing-box-windows/sing-box/sing-box.exe\nLinux: 用户目录/.local/share/sing-box-windows/sing-box/sing-box\n\n完成后点击"检查安装"按钮验证安装是否成功。',
      installPrompt: '请下载并安装内核后使用。',
    },

    // 开发者工具
    developer: {
      title: '开发者工具',
      description: '开发者调试工具和选项',
      openDevtools: '打开开发者工具',
      openDevtoolsDesc: '打开浏览器开发者工具用于调试和检查前端代码',
      open: '打开',
      opened: '开发者工具已打开',
      warning: '开发者工具主要用于开发和调试，正常使用无需开启',
    },

    // 关于
    about: {
      title: '关于',
      description: '应用程序信息和版本详情',
      system: '系统',
      license: '许可证',
    },
  },

  // 通知消息
  notification: {
    proxyModeChanged: '代理模式已更改',
    proxyModeChangeFailed: '代理模式更改失败',
    updateAvailable: '有新版本可用',
    prereleaseAvailable: '有新测试版本可用',
    updateDownloaded: '更新已下载，即将安装',
    restartAsAdmin: '以管理员重启',
    restartFailed: '重启失败',
  },
}