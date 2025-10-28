export default {
  // Common vocabulary
  common: {
    appName: 'Sing-Box Windows',
    ok: 'OK',
    cancel: 'Cancel',
    confirm: 'Confirm',
    save: 'Save',
    delete: 'Delete',
    edit: 'Edit',
    error: 'Error',
    success: 'Success',
    refresh: 'Refresh',
    unknown: 'Unknown',
    restart: 'Restart',
    search: 'Search',
    later: 'Later',
    minimizedToTray: 'Minimized to tray',
  },

  // Navigation menu
  nav: {
    navigation: 'Navigation',
    home: 'Home',
    proxy: 'Proxy',
    connections: 'Connections',
    logs: 'Logs',
    rules: 'Rules',
    subscription: 'Subscription',
    settings: 'Settings',
    expand: 'Expand',
    collapse: 'Collapse',
  },

  // Status related
  status: {
    running: 'Running',
    stopped: 'Stopped',
    starting: 'Starting',
    stopping: 'Stopping',
    restarting: 'Restarting',
    connecting: 'Connecting',
    disconnected: 'Disconnected',
  },

  // Home page
  home: {
    // Basic operations
    start: 'Start',
    stop: 'Stop',
    restart: 'Restart',
    restartAsAdmin: 'Restart as Admin',

    // Status messages
    startSuccess: 'Started successfully',
    startFailed: 'Start failed',
    stopSuccess: 'Stopped successfully',
    stopFailed: 'Stop failed',
    restartFailed: 'Restart failed',
    nodeModeChangeSuccess: 'Node mode changed successfully',
    nodeModeChangeFailed: 'Node mode change failed',

    // WebSocket status
    wsStatus: {
      connected: 'Connected',
      disconnected: 'Disconnected',
    },

    // Admin status
    adminStatus: {
      admin: 'Administrator',
      normal: 'Normal User',
    },

    // Proxy headers
    proxyHeader: {
      flowMode: 'Flow Proxy Mode',
      nodeMode: 'Node Proxy Mode',
    },

    // Status descriptions
    status: {
      startingDesc: 'Starting kernel, please wait...',
      stoppingDesc: 'Stopping kernel, please wait...',
      runningDesc: 'Kernel is running, proxy service available',
      stoppedDesc: 'Kernel stopped, proxy service unavailable',
      disconnectedDesc: 'Kernel started but connection abnormal, please check configuration',
    },

    // Traffic statistics
    traffic: {
      title: 'Traffic Statistics',
      uploadSpeed: 'Upload Speed',
      downloadSpeed: 'Download Speed',
      uploadTotal: 'Total Upload',
      downloadTotal: 'Total Download',
      memory: 'Memory Usage',
      connectionsLabel: 'connections',
    },

    // Traffic proxy mode
    proxyMode: {
      system: 'System Proxy',
      tun: 'TUN Mode',
      manual: 'Manual Mode',
      systemTip: 'Auto configure system proxy',
      tunTip: 'Use TUN virtual network card to proxy all traffic, requires admin privileges',
      manualTip: 'Requires manual configuration of system proxy',
      systemDescription: 'Automatically configure system proxy settings to provide proxy services for browsers and applications',
      tunDescription: 'Capture and proxy all system traffic through virtual network card, requires administrator privileges',
      manualDescription: 'Requires manual proxy configuration in applications and browsers',
    },

    // Node proxy mode
    nodeMode: {
      global: 'Global Mode',
      rule: 'Rule Mode',
      globalTip: 'Route all traffic through proxy',
      ruleTip: 'Intelligent traffic routing based on rules',
    },

    // Switch mode
    switchMode: 'Switch Mode',
  },

  // Proxy page
  proxy: {
    title: 'Proxy Settings',
    subtitle: 'Proxy node management and latency testing',
    search: 'Search',
    nodes: 'Nodes',
    nodeCount: 'nodes',
    testNode: 'Test',
    testing: 'Testing...',
    timeout: 'Timeout',
    testProgress: 'Test Progress:',
    switch: 'Switch',
    switchFailed: 'Switch failed',
    switchErrorMessage: 'Switch failed, please check if Sing-Box is running',
    testErrorMessage: 'Latency test failed, node may be unreachable or API not responding',
    testFailed: 'Latency test failed',
    nodeTestFailed: 'Node latency test failed',
    nodeTestComplete: 'Node latency test complete',
    groupTestComplete: 'Group latency test complete',
    batchTestComplete: 'Batch latency test complete',
    loadSuccess: 'Proxy list loaded successfully',
    loadFailed: 'Failed to get proxy list',
    loadFailedCheck: 'Failed to get proxy list, please check if Sing-Box is running',
    loadingInfo: 'Loading proxy information...',
    noProxyGroups: 'No proxy groups available',
    checkConfigOrRefresh: 'Please check configuration file or refresh page',
    currentLabel: 'Current node:',
  },

  // Connections page
  connections: {
    title: 'Connection Management',
    subtitle: 'Manage and monitor network connections',
    source: 'Source',
    destination: 'Destination',
    networkType: 'Network Type',
    rule: 'Rule',
    traffic: 'Traffic',
    activeConnections: 'Active Connections',
    refreshConnections: 'Refresh Connections',
    refreshSuccess: 'Connection list refreshed',
    refreshError: 'Failed to refresh connection list',
    noActiveConnections: 'No active network connections currently',
    adjustSearchOrFilters: 'Try adjusting search conditions or filters',
    searchPlaceholder: 'Search connections (ID, IP, domain, process)',
    networkTypeFilter: 'Network Type',
    ruleFilter: 'Rule Filter',
    noMatchingConnections2: 'No matching connections',
    matchedConnections: 'Matched Connections',
  },

  // Log page
  log: {
    title: 'Log Viewer',
    subtitle: 'Monitor system logs in real time',
    search: 'Search',
    searchLogs: 'Search logs...',
    clear: 'Clear Logs',
    copy: 'Copy Logs',
    export: 'Export Logs',
    records: 'records',
    noLogs: 'No log records',
    noLogsDesc: 'No log records currently',
    clearedSuccess: 'Logs cleared',
    copiedSuccess: 'Logs copied to clipboard',
    copyFailed: 'Copy failed',
    exportedSuccess: 'Logs exported',
    clearSearch: 'Clear Search',
    filterType: 'Filter Log Types',
    autoScroll: 'Auto Scroll',
    manualScroll: 'Manual Scroll',
    types: {
      info: 'Info',
      warning: 'Warning',
      error: 'Error',
      success: 'Success',
    },
  },

  // Rules page
  rules: {
    title: 'Rule Management',
    subtitle: 'Manage proxy routing rules',
    search: 'Search',
    add: 'Add Rule',
    type: 'Rule Type',
    content: 'Rule Content',
    targetProxy: 'Target Proxy',
    directConnect: 'Direct Connection',
    blockAction: 'Block',
    noRulesData: 'No rule data available',
    adjustSearchOrFilters: 'Please try adjusting search conditions or filter conditions',
    clickRefreshToGetRules: 'Click refresh button to get rule data',
    getRules: 'Get Rules',
    clearFilters: 'Clear Filters',
    searchPlaceholder: 'Search rule content, type, proxy...',
    ruleTypes: 'Rule Types',
    proxyTargets: 'Proxy Targets',
    totalRules: 'Total Rules',
    matchingRules: 'Matching Rules',
  },

  // Subscription page
  sub: {
    title: 'Subscription Management',
    subtitle: 'Manage your subscription configurations and proxy nodes',
    total: 'Total',
    active: 'Active',
    add: 'Add Subscription',
    edit: 'Edit Subscription',
    editConfig: 'Edit Configuration',
    editCurrentConfig: 'Edit Current Configuration',
    saveAndApply: 'Save and Apply',
    inUse: 'In Use',
    manual: 'Manual Edit',
    deleteSuccess: 'Deleted successfully',
    cannotDeleteActive: 'Cannot delete subscription in use',
    name: 'Name',
    url: 'URL',
    content: 'Content',
    namePlaceholder: 'Please enter subscription name',
    urlPlaceholder: 'Please enter subscription URL',
    manualContentPlaceholder: 'Please enter configuration content (JSON format)',
    configContentPlaceholder: 'Configuration file content (JSON format)',
    nameRequired: 'Please enter subscription name',
    urlRequired: 'Please enter subscription URL',
    contentRequired: 'Please enter configuration content',
    invalidUrl: 'Please enter valid URL',
    neverUsed: 'Never used',
    addAndUseSuccess: 'Subscription added and applied successfully',
    updateSuccess: 'Subscription updated successfully',
    operationFailed: 'Operation failed: ',
    use: 'Use',
    useSuccess: 'Applied successfully',
    useFailed: 'Application failed: ',
    copyLink: 'Copy Link',
    linkCopied: 'Link copied to clipboard',
    lastUpdate: 'Last Update',
    noSubs: 'No subscriptions',
    noSubscriptionsYet: 'You haven\'t added any subscriptions yet',
    addFirstSubscription: 'Add first subscription',
    readConfigFailed: 'Failed to read configuration: ',
    configSaved: 'Configuration saved and applied',
    saveConfigFailed: 'Failed to save configuration: ',
    useOriginal: 'Use Original Subscription',
    useOriginalConfig: 'Use original subscription directly (replace ports only)',
    useExtractedNodes: 'Extract nodes to local template',
    urlSubscription: 'URL Subscription',
    manualConfig: 'Manual Configuration',
    manualContent: 'Manual Configuration Content',
  },

  // Settings page
  setting: {
    title: 'Settings',
    subtitle: 'Manage application configuration and system settings',
    appVersion: 'App Version',
    kernelVersion: 'Kernel Version',
    notInstalled: 'Not Installed',
    newVersionFound: 'New Version Found',

    // General settings
    general: {
      title: 'General Settings',
      description: 'Configure language, network and other application options',
    },

    // Theme settings
    theme: {
      light: 'Light',
      dark: 'Dark',
    },

    // Language settings
    language: {
      title: 'Language',
      auto: 'Auto',
      changed: 'Language changed',
      changeSuccess: 'Language changed successfully',
      description: 'Select application interface display language',
    },

    // Network settings
    network: {
      ipv6: 'IPv6 Support',
      ipv4Only: 'IPv4 Only',
      ipv6Enabled: 'IPv6 Enabled',
      ipv6Desc: 'Enable or disable IPv6 network protocol support',
      ipVersionChanged: 'IP version settings changed',
      ports: 'Port Settings',
      portsDesc: 'Configure proxy and API ports',
      configure: 'Configure Ports',
      proxyPort: 'Proxy Port',
      apiPort: 'API Port',
      portSettings: 'Port Settings',
      invalidPort: 'Invalid port number, please enter a port between 1024-65535',
      portConflict: 'Proxy port and API port cannot be the same',
      restartRequired: 'Kernel restart required',
      restartDesc: 'Kernel restart required after port changes to take effect',
      portChanged: 'Port settings changed',
      portChangeSuccess: 'Port configuration updated successfully',
    },

    // Update settings
    update: {
      title: 'Update Settings',
      description: 'Configure application auto-update and version check options',
      check: 'Check for Updates',
      checkNow: 'Check Now',
      checkAgain: 'Check Again',
      checking: 'Checking...',
      autoCheck: 'Auto Check Updates',
      autoCheckDesc: 'Automatically check for application updates on startup',
      acceptPrerelease: 'Accept Pre-release Versions',
      acceptPrereleaseDesc: 'Allow receiving and installing pre-release versions (test versions)',
      current: 'Current Version',
      currentVersion: 'Current Version',
      newVersion: 'New Version',
      latestVersion: 'Latest Version',
      latest: 'Latest Version',
      hasUpdate: 'Has Update',
      confirmUpdate: 'Update now?',
      later: 'Maybe Later',
      updateNow: 'Update Now',
      downloading: 'Downloading update',
      installing: 'Installing',
      preparingDownload: 'Preparing download...',
      checkError: 'Update check failed',
      updateFailed: 'Update failed',
      alreadyLatest: 'Already latest version',
      beta: 'Beta',
      releaseNotes: 'Release Notes',
      updateNotice: 'Update Notice',
      prereleaseWarningDesc: 'Pre-release versions may contain unstable features and potential issues, recommended for testing environments only.',
      prereleaseConfirm: 'Confirm Enable Pre-release',
      prereleaseConfirmDesc: 'After enabling, you will receive pre-release version update notifications. These versions may not be stable, are you sure you want to continue?',
      prereleaseEnabled: 'Pre-release versions enabled',
      prereleaseEnabledDesc: 'You will receive pre-release version update notifications',
      prereleaseDisabled: 'Pre-release versions disabled',
      prereleaseDisabledDesc: 'You will only receive stable version update notifications',
    },

    // Auto-start settings
    autoStart: {
      app: 'Startup with System',
      appDesc: 'Automatically run application on system startup',
      kernel: 'Start Kernel',
      kernelDesc: 'Automatically run kernel service on application startup',
    },

    // Kernel auto-start
    kernelAutoStart: {
      enabled: 'Auto-start kernel enabled',
      disabled: 'Auto-start kernel disabled',
      enableSuccess: 'Kernel will start automatically when application starts',
      disableSuccess: 'Kernel will not start automatically when application starts',
    },

    // Startup settings
    startup: {
      title: 'Startup Settings',
      description: 'Configure application startup and auto-run options',
      enabled: 'Startup with system enabled',
      disabled: 'Startup with system disabled',
      enableSuccess: 'Startup with system setting successful',
      disableSuccess: 'Startup with system disabled',
    },

    // Kernel management
    kernel: {
      title: 'Kernel Management',
      description: 'Manage Sing-Box kernel version and download',
      download: 'Download Kernel',
      redownload: 'Redownload',
      update: 'Update Kernel',
      checkInstall: 'Check Installation',
      downloadComplete: 'Kernel download complete!',
      installSuccess: 'Kernel installation verification successful!',
      installFailed: 'No valid kernel file detected',
      downloadFailed: 'Download failed',
      preparingDownload: 'Preparing download...',
      downloading: 'Downloading kernel',
      downloadingDescription: 'Kernel file is downloading, please wait...',
      manualDownload: 'Manual Download',
      manualDownloadTitle: 'Manual Download Instructions',
      manualDownloadGuide: 'Please follow these steps:\n1. Visit https://github.com/SagerNet/sing-box/releases/latest\n2. Download the corresponding system version of sing-box\n3. Place the extracted executable file in the following directory:\nWindows: User directory/AppData/Local/sing-box-windows/sing-box/sing-box.exe\nLinux: User directory/.local/share/sing-box-windows/sing-box/sing-box\n\nAfter completion, click "Check Installation" button to verify installation.',
      installPrompt: 'Please download and install kernel before use.',
    },

    // Developer tools
    developer: {
      title: 'Developer Tools',
      description: 'Developer debugging tools and options',
      openDevtools: 'Open Developer Tools',
      openDevtoolsDesc: 'Open browser developer tools for debugging and inspecting frontend code',
      open: 'Open',
      opened: 'Developer tools opened',
      warning: 'Developer tools are mainly for development and debugging, no need to enable for normal use',
    },

    // About
    about: {
      title: 'About',
      description: 'Application information and version details',
      system: 'System',
      license: 'License',
    },
  },

  // Tray menu
  tray: {
    kernel: 'Kernel',
    show: 'Show Window',
    start: 'Start Kernel',
    stop: 'Stop Kernel',
    quit: 'Quit Application',
  },

  // Proxy related
  proxyMode: {
    currentMode: 'Current Mode',
  },

  // Notification messages
  notification: {
    proxyModeChanged: 'Proxy mode changed',
    proxyModeChangeFailed: 'Proxy mode change failed',
    updateAvailable: 'New version available',
    prereleaseAvailable: 'New pre-release version available',
    updateDownloaded: 'Update downloaded, installing soon',
    restartAsAdmin: 'Restart as Admin',
    restartFailed: 'Restart failed',
  },
}