export default {
  // 通用词汇
  common: {
    appName: 'Sing-Box Windows',
    ok: 'OK',
    cancel: 'キャンセル',
    confirm: '確認',
    save: '保存',
    delete: '削除',
    edit: '編集',
    error: 'エラー',
    success: '成功',
    refresh: '更新',
    unknown: '不明',
    restart: '再起動',
    search: '検索',
    later: '後で',
    minimizedToTray: 'トレイに最小化されました',
  },

  // 导航菜单
  nav: {
    navigation: 'ナビゲーション',
    home: 'ホーム',
    proxy: 'プロキシ',
    connections: '接続',
    logs: 'ログ',
    rules: 'ルール',
    subscription: 'サブスクリプション',
    settings: '設定',
    expand: '展開',
    collapse: '折りたたみ',
  },

  // 状态相关
  status: {
    running: '実行中',
    stopped: '停止中',
    starting: '起動中',
    stopping: '停止中',
    restarting: '再起動中',
    connecting: '接続中',
    disconnected: '切断',
  },

  // 首页
  home: {
    // 基础操作
    start: '起動',
    stop: '停止',
    restart: '再起動',
    restartAsAdmin: '管理者として再起動',

    // 状态信息
    startSuccess: '起動成功',
    startFailed: '起動失敗',
    stopSuccess: '停止成功',
    stopFailed: '停止失敗',
    restartFailed: '再起動失敗',
    nodeModeChangeSuccess: 'ノードモードの切り替え成功',
    nodeModeChangeFailed: 'ノードモードの切り替え失敗',

    // WebSocket状态
    wsStatus: {
      connected: '接続済み',
      disconnected: '未接続',
    },

    // 管理员状态
    adminStatus: {
      admin: '管理者',
      normal: '一般ユーザー',
    },

    // 代理头部
    proxyHeader: {
      flowMode: 'トラフィックプロキシモード',
      nodeMode: 'ノードプロキシモード',
    },

    // 状态描述
    status: {
      startingDesc: 'カーネルを起動中です、お待ちください...',
      stoppingDesc: 'カーネルを停止中です、お待ちください...',
      runningDesc: 'カーネルが実行中です、プロキシサービスが利用可能です',
      stoppedDesc: 'カーネルが停止しています、プロキシサービスは利用できません',
      disconnectedDesc: 'カーネルは起動していますが接続に異常があります、設定を確認してください',
    },

    // 流量统计
    traffic: {
      title: 'トラフィック統計',
      uploadSpeed: 'アップロード速度',
      downloadSpeed: 'ダウンロード速度',
      uploadTotal: '総アップロード',
      downloadTotal: '総ダウンロード',
      memory: 'メモリ使用量',
      connectionsLabel: '接続',
    },

    // 流量代理模式
    proxyMode: {
      system: 'システムプロキシ',
      tun: 'TUNモード',
      manual: '手動モード',
      systemTip: '自動的にシステムプロキシを設定する',
      tunTip: 'TUN仮想ネットワークカードですべてのトラフィックをプロキシ、管理者権限が必要',
      manualTip: '手動でシステムプロキシを設定する必要があります',
      systemDescription: 'ウェブブラウザやアプリケーションのシステムプロキシ設定を自動的に構成します',
      tunDescription: '仮想ネットワークカードを通じてすべてのシステムトラフィックをキャプチャしプロキシします、管理者権限が必要です',
      manualDescription: 'アプリケーションやブラウザでプロキシ設定を手動で構成します',
    },

    // 节点代理模式
    nodeMode: {
      global: 'グローバルモード',
      rule: 'ルールモード',
      globalTip: 'すべてのトラフィックをプロキシ経由',
      ruleTip: 'ルールベースのスマート分割',
    },

    // 切换模式
    switchMode: 'モード切り替え',
  },

  // 代理页面
  proxy: {
    title: 'プロキシ設定',
    subtitle: 'プロキシノード管理と遅延テスト',
    search: '検索',
    nodes: 'ノード',
    nodeCount: 'ノード',
    testNode: 'テスト',
    testing: 'テスト中...',
    timeout: 'タイムアウト',
    testProgress: 'テスト進捗:',
    switch: '切り替え',
    switchFailed: '切り替え失敗',
    switchErrorMessage: '切り替え失敗、Sing-Boxが起動しているか確認してください',
    testErrorMessage: '遅延テスト失敗、ノードが到達不能またはAPIが応答していません',
    testFailed: '遅延テスト失敗',
    nodeTestFailed: 'ノード遅延テスト失敗',
    nodeTestComplete: 'ノード遅延テスト完了',
    groupTestComplete: 'グループ遅延テスト完了',
    batchTestComplete: 'バッチ遅延テスト完了',
    loadSuccess: 'プロキシリスト読み込み成功',
    loadFailed: 'プロキシリスト取得失敗',
    loadFailedCheck: 'プロキシリスト取得失敗、Sing-Boxが起動しているか確認してください',
    loadingInfo: 'プロキシ情報を読み込み中...',
    noProxyGroups: 'プロキシグループがありません',
    checkConfigOrRefresh: '設定ファイルを確認するかページを更新してください',
    currentLabel: '現在のノード:',
  },

  // 连接页面
  connections: {
    title: '接続管理',
    subtitle: 'ネットワーク接続の管理と監視',
    source: '送信元',
    destination: '宛先',
    networkType: 'ネットワークタイプ',
    rule: 'ルール',
    traffic: 'トラフィック',
    activeConnections: 'アクティブ接続',
    refreshConnections: '接続を更新',
    refreshSuccess: '接続リストが更新されました',
    refreshError: '接続リストの更新に失敗しました',
    noActiveConnections: '現在アクティブなネットワーク接続がありません',
    adjustSearchOrFilters: '検索条件やフィルターを調整してください',
    searchPlaceholder: '接続を検索 (ID、IP、ドメイン、プロセス)',
    networkTypeFilter: 'ネットワークタイプ',
    ruleFilter: 'ルールフィルタ',
    noMatchingConnections2: '一致する接続がありません',
    matchedConnections: '一致接続',
  },

  // 日志页面
  log: {
    title: 'ログビューア',
    subtitle: 'システムログをリアルタイムで監視',
    search: '検索',
    searchLogs: 'ログを検索...',
    clear: 'ログをクリア',
    copy: 'ログをコピー',
    export: 'ログをエクスポート',
    records: '件',
    noLogs: 'ログ記録がありません',
    noLogsDesc: '現在ログ記録がありません',
    clearedSuccess: 'ログがクリアされました',
    copiedSuccess: 'ログがクリップボードにコピーされました',
    copyFailed: 'コピーに失敗しました',
    exportedSuccess: 'ログがエクスポートされました',
    clearSearch: '検索をクリア',
    filterType: 'ログタイプをフィルタ',
    autoScroll: '自動スクロール',
    manualScroll: '手動スクロール',
    types: {
      info: '情報',
      warning: '警告',
      error: 'エラー',
      success: '成功',
    },
  },

  // 规则页面
  rules: {
    title: 'ルール管理',
    subtitle: 'プロキシルーティングルールの管理',
    search: '検索',
    add: 'ルールを追加',
    type: 'ルールタイプ',
    content: 'ルール内容',
    targetProxy: 'ターゲットプロキシ',
    directConnect: 'ローカル直接接続',
    blockAction: 'ブロック',
    noRulesData: 'ルールデータがありません',
    adjustSearchOrFilters: '検索条件やフィルター条件を調整してください',
    clickRefreshToGetRules: '更新ボタンをクリックしてルールデータを取得してください',
    getRules: 'ルールを取得',
    clearFilters: 'フィルタをクリア',
    searchPlaceholder: 'ルール内容、タイプ、プロキシを検索...',
    ruleTypes: 'ルールタイプ',
    proxyTargets: 'プロキシターゲット',
    totalRules: '総ルール',
    matchingRules: '一致ルール',
  },

  // 订阅页面
  sub: {
    title: 'サブスクリプション管理',
    subtitle: 'サブスクリプション設定とプロキシノードを管理',
    total: '合計',
    active: 'アクティブ',
    add: 'サブスクリプションを追加',
    edit: 'サブスクリプションを編集',
    editConfig: '設定を編集',
    editCurrentConfig: '現在の設定を編集',
    saveAndApply: '保存して適用',
    inUse: '使用中',
    manual: '手動編集',
    deleteSuccess: '削除成功',
    cannotDeleteActive: '使用中のサブスクリプションは削除できません',
    name: '名前',
    url: 'URL',
    content: 'コンテンツ',
    namePlaceholder: 'サブスクリプション名を入力してください',
    urlPlaceholder: 'サブスクリプションURLを入力してください',
    manualContentPlaceholder: '設定コンテンツを入力してください（JSON形式）',
    configContentPlaceholder: '設定ファイルコンテンツ（JSON形式）',
    nameRequired: 'サブスクリプション名を入力してください',
    urlRequired: 'サブスクリプションURLを入力してください',
    contentRequired: '設定コンテンツを入力してください',
    invalidUrl: '有効なURLを入力してください',
    neverUsed: '未使用',
    addAndUseSuccess: 'サブスクリプションの追加と適用が成功しました',
    updateSuccess: 'サブスクリプションの更新が成功しました',
    operationFailed: '操作失敗: ',
    use: '使用',
    useSuccess: '適用成功',
    useFailed: '適用失敗: ',
    copyLink: 'リンクをコピー',
    linkCopied: 'リンクがクリップボードにコピーされました',
    lastUpdate: '最終更新',
    noSubs: 'サブスクリプションがありません',
    noSubscriptionsYet: 'まだサブスクリプションを追加していません',
    addFirstSubscription: '最初のサブスクリプションを追加',
    readConfigFailed: '設定の読み込みに失敗しました: ',
    configSaved: '設定が保存され適用されました',
    saveConfigFailed: '設定の保存に失敗しました: ',
    useOriginal: '元のサブスクリプションを使用',
    useOriginalConfig: '元のサブスクリプションを直接使用（ポートのみ置換）',
    useExtractedNodes: 'ノードをローカルテンプレートに抽出',
    urlSubscription: 'URLサブスクリプション',
    manualConfig: '手動設定',
    manualContent: '手動設定コンテンツ',
  },

  // 设置页面
  setting: {
    title: '設定',
    subtitle: 'アプリケーション設定とシステム設定の管理',
    appVersion: 'アプリバージョン',
    kernelVersion: 'カーネルバージョン',
    notInstalled: 'インストールされていません',
    newVersionFound: '新しいバージョンが見つかりました',

    // 通用设置
    general: {
      title: '一般設定',
      description: '言語、ネットワーク、その他のアプリケーションオプションを設定',
    },

    // 主题设置
    theme: {
      light: 'ライト',
      dark: 'ダーク',
    },

    // 语言设置
    language: {
      title: '言語',
      auto: '自動',
      changed: '言語が変更されました',
      changeSuccess: '言語が正常に変更されました',
      description: 'アプリケーションインターフェースの表示言語を選択',
    },

    // 网络设置
    network: {
      ipv6: 'IPv6 サポート',
      ipv4Only: 'IPv4 のみ',
      ipv6Enabled: 'IPv6が有効',
      ipv6Desc: 'IPv6ネットワークプロトコルサポートを有効または無効にする',
      ipVersionChanged: 'IPバージョン設定が変更されました',
      ports: 'ポート設定',
      portsDesc: 'プロキシとAPIポートを設定',
      configure: 'ポートを設定',
      proxyPort: 'プロキシポート',
      apiPort: 'APIポート',
      portSettings: 'ポート設定',
      invalidPort: '無効なポート番号、1024-65535の間のポートを入力してください',
      portConflict: 'プロキシポートとAPIポートは同じにできません',
      restartRequired: 'カーネルの再起動が必要',
      restartDesc: 'ポート変更後、カーネルの再起動が必要です',
      portChanged: 'ポート設定が変更されました',
      portChangeSuccess: 'ポート設定が正常に更新されました',
    },

    // 更新设置
    update: {
      title: '更新設定',
      description: 'アプリの自動更新とバージョンチェックオプションを設定',
      check: '更新を確認',
      autoCheck: '自動更新チェック',
      autoCheckDesc: '起動時にアプリの更新を自動的にチェック',
      acceptPrerelease: 'プレリリースバージョンを受け取る',
      acceptPrereleaseDesc: 'プレリリースバージョン（テストバージョン）の受信とインストールを許可',
      current: '現在のバージョン',
      newVersion: '新しいバージョン',
      confirmUpdate: '今すぐ更新しますか？',
      later: '後で',
      updateNow: '今すぐ更新',
      downloading: '更新をダウンロード中',
      installing: 'インストール中',
      preparingDownload: 'ダウンロード準備中...',
      checkError: '更新チェック失敗',
      updateFailed: '更新失敗',
      alreadyLatest: '既に最新バージョンです',
      beta: 'ベータ',
      releaseNotes: 'リリースノート',
      updateNotice: '更新通知',
      prereleaseWarningDesc: 'テストバージョンには不安定な機能や潜在的な問題が含まれる可能性があります。テスト環境でのみ使用することをお勧めします。',
      prereleaseConfirm: 'プレリリースバージョンを有効にする確認',
      prereleaseConfirmDesc: '有効にすると、プレリリースバージョンの更新通知を受信します。これらのバージョンは不安定な可能性がありますが、続行しますか？',
      prereleaseEnabled: 'プレリリースバージョンが有効になりました',
      prereleaseEnabledDesc: 'プレリリースバージョンの更新通知を受信します',
      prereleaseDisabled: 'プレリリースバージョンが無効になりました',
      prereleaseDisabledDesc: '安定版の更新通知のみを受信します',
    },

    // 自启动设置
    autoStart: {
      app: 'システム起動時に起動',
      appDesc: 'システム起動時にアプリを自動的に実行',
      kernel: 'カーネルを起動',
      kernelDesc: 'アプリ起動時にカーネルサービスを自動的に実行',
    },

    // 内核自动启动
    kernelAutoStart: {
      enabled: 'カーネル自動起動が有効',
      disabled: 'カーネル自動起動が無効',
      enableSuccess: 'アプリ起動時にカーネルが自動的に起動します',
      disableSuccess: 'アプリ起動時にカーネルは自動的に起動しません',
    },

    // 启动设置
    startup: {
      title: '起動設定',
      description: 'アプリ起動と自動実行オプションを設定',
      enabled: 'システム起動が有効',
      disabled: 'システム起動が無効',
      enableSuccess: 'システム起動設定が成功しました',
      disableSuccess: 'システム起動が無効になりました',
    },

    // 内核管理
    kernel: {
      title: 'カーネル管理',
      description: 'Sing-Boxカーネルバージョンとダウンロードを管理',
      download: 'カーネルをダウンロード',
      redownload: '再ダウンロード',
      update: 'カーネルを更新',
      checkInstall: 'インストールを確認',
      downloadComplete: 'カーネルダウンロード完了！',
      installSuccess: 'カーネルインストール確認成功！',
      installFailed: '有効なカーネルファイルが検出されませんでした',
      downloadFailed: 'ダウンロード失敗',
      preparingDownload: 'ダウンロード準備中...',
      downloading: 'カーネルをダウンロード中',
      downloadingDescription: 'カーネルファイルをダウンロード中です、お待ちください...',
      manualDownload: '手動ダウンロード',
      manualDownloadTitle: '手動ダウンロード説明',
      manualDownloadGuide: '以下の手順に従ってください：\n1. https://github.com/SagerNet/sing-box/releases/latest にアクセス\n2. 対応するシステムバージョンのsing-boxをダウンロード\n3. 解凍した実行ファイルを以下のディレクトリに配置：\nWindows: ユーザーディレクトリ/AppData/Local/sing-box-windows/sing-box/sing-box.exe\nLinux: ユーザーディレクトリ/.local/share/sing-box-windows/sing-box/sing-box\n\n完了後、「インストール確認」ボタンをクリックしてインストールが成功したか確認してください。',
      installPrompt: '使用前にカーネルをダウンロードしてインストールしてください。',
    },

    // 开发者工具
    developer: {
      title: '開発者ツール',
      description: '開発者デバッグツールとオプション',
      openDevtools: '開発者ツールを開く',
      openDevtoolsDesc: 'フロントエンドコードのデバッグと検査のためのブラウザ開発者ツールを開きます',
      open: '開く',
      opened: '開発者ツールが開かれました',
      warning: '開発者ツールは主に開発とデバッグ用で、通常の使用では有効にする必要はありません',
    },

    // 关于
    about: {
      title: 'について',
      description: 'アプリケーション情報とバージョン詳細',
      system: 'システム',
      license: 'ライセンス',
    },
  },

  // トレイメニュー
  tray: {
    kernel: 'カーネル',
    show: 'ウィンドウを表示',
    start: 'カーネルを起動',
    stop: 'カーネルを停止',
    quit: 'アプリケーションを終了',
  },

  // プロキシ関連
  proxyMode: {
    currentMode: '現在のモード',
  },

  // 通知メッセージ
  notification: {
    proxyModeChanged: 'プロキシモードが変更されました',
    proxyModeChangeFailed: 'プロキシモードの変更に失敗しました',
    updateAvailable: '新しいバージョンが利用可能です',
    prereleaseAvailable: '新しいプレリリースバージョンが利用可能です',
    updateDownloaded: '更新がダウンロードされました、まもなくインストールされます',
    restartAsAdmin: '管理者として再起動',
    restartFailed: '再起動失敗',
  },
}