export default {
  common: {
    ok: '确定',
    cancel: '取消',
    confirm: '确认',
    save: '保存',
    delete: '删除',
    edit: '编辑',
    loading: '加载中...',
    unknown: '未知',
    error: '错误',
    success: '成功',
    refresh: '刷新',
    on: '开',
    off: '关',
  },
  nav: {
    home: '首页',
    proxy: '代理',
    sub: '订阅',
    log: '日志',
    setting: '设置',
    rules: '规则',
    connections: '连接',
  },
  home: {
    status: {
      running: '运行中',
      stopped: '已停止',
      starting: '启动中...',
      stopping: '停止中...',
      restarting: '重启中...',
    },
    proxyMode: {
      system: '系统代理',
      tun: 'TUN模式',
    },
    switchMode: '切换模式',
    start: '启动',
    stop: '停止',
    restart: '重启',
    traffic: {
      uploadSpeed: '上传速度',
      downloadSpeed: '下载速度',
      uploadTotal: '上传总流量',
      downloadTotal: '下载总流量',
      memory: '内存占用',
      connections: '活动连接',
    },
  },
  chart: {
    startUpdateTimer: '启动图表更新定时器',
    resetRefresh: '重置并刷新图表',
    themeChanged: '主题变化，重绘图表',
    windowResized: '窗口大小变化，重新绘制图表',
  },
  proxy: {
    title: '代理设置',
    selectNode: '选择节点',
    testLatency: '测试延迟',
    search: '搜索',
    noNodes: '没有可用节点',
    currentNode: '当前节点',
    nodeCount: '个节点',
    speedTest: '测延迟',
    notTested: '未测试',
    inUse: '使用中',
    switch: '切换',
    modeSwitchTip: '点击切换代理模式',
    refreshList: '刷新代理列表',
    switchTo: '切换到',
    switchModeConfirm: '切换代理模式需要重启内核才能生效。确定要切换并重启内核吗？',
    confirmSwitch: '确认切换',
    testProgress: '测试进度:',
    groupTestComplete: '组延迟测试完成',
    testFailed: '延迟测试失败',
    batchTestComplete: '批量延迟测试完成',
    loadSuccess: '代理列表加载成功',
    loadFailed: '获取代理列表失败',
    loadFailedCheck: '获取代理列表失败，请检查Sing-Box是否已启动',
    testErrorMessage: '延迟测试失败，可能是节点无法连接或API未响应',
    switchSuccess: '已切换 {group} 到 {proxy}',
    switchFailed: '切换失败',
    switchErrorMessage: '切换失败，请检查Sing-Box是否已启动',
    currentMode: '当前代理模式:',
    getModeError: '获取代理模式失败',
    modeChangeFailed: '切换代理模式失败',
    modeChangeError: '切换代理模式失败',
    modeChangeSuccess: '已切换到{mode}并重启内核',
    mode: {
      global: '全局模式',
      rule: '规则模式',
      tun: 'TUN模式',
      unknown: '未知模式',
    },
  },
  sub: {
    title: '订阅管理',
    count: '个订阅',
    add: '添加订阅',
    edit: '编辑订阅',
    editConfig: '编辑配置',
    editCurrentConfig: '编辑当前配置',
    saveAndApply: '保存并应用',
    inUse: '使用中',
    manual: '手动编辑',
    deleteConfirm: '确定要删除这个订阅吗？',
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
    addSuccess: '添加订阅成功',
    updateSuccess: '更新订阅成功',
    operationFailed: '操作失败: ',
    use: '使用',
    useAgain: '重新使用',
    useSuccess: '应用成功',
    useFailed: '应用失败: ',
    copyLink: '复制链接',
    linkCopied: '链接已复制到剪贴板',
    lastUpdate: '更新时间',
    noSubs: '暂无订阅',
    urlAdd: '从链接添加',
    manualAdd: '手动添加',
    readConfigFailed: '读取配置失败: ',
    configSaved: '配置已保存并应用',
    saveConfigFailed: '保存配置失败: ',
    deleteSuccess: '删除成功',
    useRules: '使用规则集',
    useSubRules: '使用订阅规则集',
    useDefaultRules: '使用默认规则集',
  },
  log: {
    title: '日志查看',
    level: '日志级别',
    clear: '清空日志',
    copy: '复制日志',
    export: '导出日志',
    search: '搜索',
    noLogs: '暂无日志记录',
    records: '条记录',
    autoScroll: '自动滚动',
    manualScroll: '手动滚动',
    filterType: '筛选日志类型',
  },
  setting: {
    title: '设置',
    general: '常规',
    theme: {
      title: '主题',
      light: '浅色',
      dark: '深色',
      system: '跟随系统',
    },
    language: {
      title: '语言',
      auto: '自动',
    },
    update: {
      title: '更新',
      check: '检查更新',
      auto: '自动检查更新',
      current: '当前版本',
      newVersion: '新版本',
      confirmUpdate: '是否立即更新？',
      later: '下次再说',
      updateNow: '立即更新',
      downloading: '正在下载更新',
      checkError: '检查更新失败',
      updateError: '更新失败: ',
      newVersionFound: '发现新版本：{version}',
      alreadyLatest: '当前已是最新版本',
    },
    autoStart: {
      title: '自动启动',
      app: '开机启动应用',
      kernel: '启动应用时自动启动内核',
    },
    advanced: {
      title: '高级',
      reset: '重置所有设置',
      resetConfirm: '确认重置所有设置？此操作不可撤销。',
    },
    kernel: {
      title: '内核管理',
      currentVersion: '当前版本：',
      notInstalled: '未安装内核',
      newVersion: '新版本：',
      newVersionFound: '发现新版本',
      updateTip: '有新版本的内核可供下载，建议更新以获得更好的体验。',
      installPrompt: '请下载并安装内核后使用。',
      downloadNew: '下载新版本',
      redownload: '重新下载当前版本',
      download: '下载内核',
      manualDownload: '手动下载',
      checkInstall: '检查安装',
      downloadFailed: '下载失败',
      preparingDownload: '准备下载...',
      downloadComplete: '内核下载完成！',
      installSuccess: '内核安装验证成功！',
      installFailed: '未检测到有效的内核文件',
      manualDownloadTitle: '手动下载说明',
      manualDownloadGuide:
        '请按照以下步骤操作：\n1. 访问 https://github.com/SagerNet/sing-box/releases/latest\n2. 下载对应系统版本的 sing-box\n3. 将解压后的 sing-box.exe 放置在以下目录：\n用户目录/AppData/Local/sing-box-windows/sing-box/\n\n完成后点击"检查安装"按钮验证安装是否成功。',
    },
    startup: {
      title: '启动设置',
      bootTip: '应用将在系统启动时自动运行',
      manualTip: '应用需要手动启动',
      autoKernelTip: '应用启动时将自动启动内核',
      manualKernelTip: '需要手动启动内核',
      enableSuccess: '已设置开机启动',
      disableSuccess: '已关闭开机启动',
    },
    network: {
      ipv6: 'IPv6优先',
      preferIpv6: '优先使用IPv6连接',
      onlyIpv4: '仅使用IPv4连接',
    },
    about: {
      title: '关于',
      appVersion: '应用版本',
      kernelVersion: '内核版本',
      system: '系统',
      license: '开源协议',
      website: '官网',
    },
    error: {
      appDataDir: '获取应用数据目录失败:',
    },
  },
  rules: {
    title: '规则管理',
    add: '添加规则',
    type: '规则类型',
    content: '规则内容',
    action: '动作',
    noRules: '暂无规则数据',
    targetProxy: '目标代理',
    directConnect: '本地直连',
    manualSwitch: '手动切换',
    autoSelect: '自动选择',
    logData: '规则数据:',
    fetchSuccess: '成功获取 {count} 条规则',
    fetchError: '获取规则失败',
    fetchErrorFormat: '获取规则失败：返回数据格式错误',
  },
  connections: {
    title: '连接管理',
    source: '来源',
    destination: '目标',
    duration: '持续时间',
    upload: '上传',
    download: '下载',
    noConnections: '暂无连接',
    activeConnections: '活跃连接',
    uploadTotal: '上传总流量',
    downloadTotal: '下载总流量',
    startTime: '开始时间',
    networkType: '网络/类型',
    rule: '规则',
    ip: 'IP',
    port: '端口',
    host: '主机',
  },
  tray: {
    show: '显示窗口',
    hide: '隐藏窗口',
    start: '启动服务',
    stop: '停止服务',
    quit: '退出',
    kernel: '内核',
  },
  notification: {
    kernelStarted: '内核已启动',
    kernelStopped: '内核已停止',
    kernelRestarted: '内核已重启',
    updateAvailable: '发现新版本',
    updateDownloaded: '更新已下载，即将安装',
    updateFailed: '更新失败',
  },
}
