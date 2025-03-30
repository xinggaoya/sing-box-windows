export default {
  common: {
    ok: 'OK',
    cancel: 'キャンセル',
    confirm: '確認',
    save: '保存',
    delete: '削除',
    edit: '編集',
    loading: '読み込み中...',
    unknown: '不明',
    error: 'エラー',
    success: '成功',
    refresh: '更新',
    on: 'オン',
    off: 'オフ',
  },
  nav: {
    home: 'ホーム',
    proxy: 'プロキシ',
    sub: '購読',
    log: 'ログ',
    setting: '設定',
    rules: 'ルール',
    connections: '接続',
  },
  home: {
    status: {
      running: '実行中',
      stopped: '停止',
      starting: '起動中...',
      stopping: '停止中...',
      restarting: '再起動中...',
    },
    proxyMode: {
      system: 'システムプロキシ',
      tun: 'TUNモード',
    },
    switchMode: 'モード切替',
    start: '開始',
    stop: '停止',
    restart: '再起動',
    traffic: {
      uploadSpeed: 'アップロード速度',
      downloadSpeed: 'ダウンロード速度',
      uploadTotal: '総アップロード',
      downloadTotal: '総ダウンロード',
      memory: 'メモリ',
      connections: 'アクティブ接続',
    },
  },
  chart: {
    startUpdateTimer: 'チャート更新タイマー開始',
    resetRefresh: 'チャートのリセットと更新',
    themeChanged: 'テーマが変更されました、チャートを再描画',
    windowResized: 'ウィンドウサイズが変更されました、チャートを再描画',
  },
  proxy: {
    title: 'プロキシ設定',
    selectNode: 'ノード選択',
    testLatency: '遅延テスト',
    search: '検索',
    noNodes: '利用可能なノードがありません',
    currentNode: '現在のノード',
    nodeCount: 'ノード',
    speedTest: 'テスト',
    notTested: 'テストなし',
    inUse: '使用中',
    switch: '切替',
    modeSwitchTip: 'クリックでプロキシモードを切り替え',
    refreshList: 'プロキシリストを更新',
    switchTo: '切り替え: ',
    switchModeConfirm:
      'プロキシモードの切り替えには、カーネルの再起動が必要です。切り替えてカーネルを再起動しますか？',
    confirmSwitch: '切り替え確認',
    testProgress: 'テスト進捗:',
    groupTestComplete: 'グループ速度テスト完了',
    testFailed: '速度テスト失敗',
    batchTestComplete: 'バッチ速度テスト完了',
    loadSuccess: 'プロキシリスト読み込み成功',
    loadFailed: 'プロキシリスト取得失敗',
    loadFailedCheck: 'プロキシリスト取得失敗、Sing-Boxが起動しているか確認してください',
    testErrorMessage: '速度テスト失敗、ノードが到達不能またはAPIが応答していません',
    switchSuccess: '{group}を{proxy}に切り替えました',
    switchFailed: '切り替え失敗',
    switchErrorMessage: '切り替え失敗、Sing-Boxが起動しているか確認してください',
    currentMode: '現在のプロキシモード:',
    getModeError: 'プロキシモード取得失敗',
    modeChangeFailed: 'プロキシモード切り替え失敗',
    modeChangeError: 'プロキシモード切り替え失敗',
    modeChangeSuccess: '{mode}に切り替えてカーネルを再起動しました',
    mode: {
      global: 'グローバルモード',
      rule: 'ルールモード',
      tun: 'TUNモード',
      unknown: '不明なモード',
    },
  },
  sub: {
    title: '購読',
    add: '購読追加',
    url: 'URL',
    name: '名前',
    update: '更新',
    updateAll: 'すべて更新',
    lastUpdate: '最終更新',
    noSubs: '購読なし',
    import: '購読インポート',
    export: '購読エクスポート',
    addSuccess: '購読追加成功',
    addFailed: '購読追加失敗',
    updateSuccess: '購読更新成功',
    updateFailed: '購読更新失敗',
    deleteConfirm: 'この購読を削除してもよろしいですか？',
    deleteSuccess: '購読削除完了',
    deleteFailed: '購読削除失敗',
    invalidUrl: '無効な購読URL',
    duplicateName: '購読名が既に存在します',
    duplicateUrl: '購読URLが既に存在します',
    convertSuccess: '変換成功',
    convertFailed: '変換失敗',
    importSuccess: 'インポート成功',
    importFailed: 'インポート失敗',
    exportSuccess: 'エクスポート成功',
    exportFailed: 'エクスポート失敗',
    count: '購読',
    inUse: '使用中',
    manual: '手動',
    copyLink: 'リンクコピー',
    edit: '購読編集',
    editConfig: '現在の設定を編集',
    neverUsed: '未使用',
    useAgain: '再使用',
    use: '使用',
    namePlaceholder: '購読名を入力',
    urlAdd: 'URL追加',
    urlPlaceholder: '購読URLを入力',
    manualAdd: '手動編集',
    content: 'コンテンツ',
    manualContentPlaceholder: '設定内容を入力（JSON形式）',
    editCurrentConfig: '現在の設定を編集',
    configContentPlaceholder: '設定内容（JSON形式）',
    saveAndApply: '保存して適用',
    nameRequired: '購読名を入力してください',
    urlRequired: '購読URLを入力してください',
    contentRequired: '設定内容を入力してください',
    operationFailed: '操作失敗: ',
    cannotDeleteActive: '現在アクティブな購読は削除できません',
    useSuccess: '購読を適用しました',
    useFailed: '購読の適用に失敗しました: ',
    linkCopied: 'リンクをクリップボードにコピーしました',
    readConfigFailed: '設定の読み込みに失敗しました: ',
    configSaved: '設定を保存して適用しました',
    saveConfigFailed: '設定の保存に失敗しました: ',
  },
  log: {
    title: 'ログ',
    level: 'ログレベル',
    clear: 'ログクリア',
    copy: 'ログコピー',
    export: 'ログエクスポート',
    search: '検索',
    noLogs: 'ログがありません',
    records: '記録',
    autoScroll: '自動スクロール',
    manualScroll: '手動スクロール',
    filterType: 'ログタイプフィルター',
  },
  setting: {
    title: '設定',
    general: '一般',
    theme: {
      title: 'テーマ',
      light: 'ライト',
      dark: 'ダーク',
      system: 'システム',
    },
    language: {
      title: '言語',
      auto: '自動',
    },
    update: {
      title: '更新',
      check: 'アップデートを確認',
      auto: '自動更新確認',
      current: '現在のバージョン',
      newVersion: '新バージョン',
      confirmUpdate: '今すぐ更新しますか？',
      later: '後で',
      updateNow: '今すぐ更新',
      downloading: '更新をダウンロード中',
      checkError: '更新の確認に失敗しました',
      updateError: '更新失敗: ',
      newVersionFound: '新バージョンが見つかりました: {version}',
      alreadyLatest: '既に最新バージョンです',
    },
    autoStart: {
      title: '自動起動',
      app: '起動時にアプリを開始',
      kernel: 'アプリ起動時にカーネルを自動開始',
    },
    advanced: {
      title: '詳細設定',
      reset: 'すべての設定をリセット',
      resetConfirm: 'すべての設定をリセットしてもよろしいですか？この操作は元に戻せません。',
    },
    kernel: {
      title: 'カーネル管理',
      currentVersion: '現在のバージョン: ',
      notInstalled: 'カーネルがインストールされていません',
      newVersion: '新バージョン: ',
      newVersionFound: '新バージョンが見つかりました',
      updateTip:
        'カーネルの新バージョンがダウンロード可能です。より良い体験のためにアップデートをお勧めします。',
      installPrompt: '使用前にカーネルをダウンロードしてインストールしてください。',
      downloadNew: '新バージョンをダウンロード',
      redownload: '現在のバージョンを再ダウンロード',
      download: 'カーネルをダウンロード',
      manualDownload: '手動ダウンロード',
      checkInstall: 'インストールを確認',
      downloadFailed: 'ダウンロード失敗',
      preparingDownload: 'ダウンロード準備中...',
      downloadComplete: 'カーネルのダウンロードが完了しました！',
      installSuccess: 'カーネルのインストールが正常に確認されました！',
      installFailed: '有効なカーネルファイルが検出されませんでした',
      manualDownloadTitle: '手動ダウンロードガイド',
      manualDownloadGuide:
        '次の手順に従ってください：\n1. https://github.com/SagerNet/sing-box/releases/latest にアクセス\n2. お使いのシステム用のsing-boxバージョンをダウンロード\n3. 解凍したsing-box.exeを次のディレクトリに配置：\nユーザーディレクトリ/AppData/Local/sing-box-windows/sing-box/\n\n完了後「インストールを確認」ボタンをクリックしてインストールを確認してください。',
    },
    startup: {
      title: '起動設定',
      bootTip: 'アプリケーションはシステム起動時に自動的に実行されます',
      manualTip: 'アプリケーションは手動で起動する必要があります',
      autoKernelTip: 'カーネルはアプリケーション起動時に自動的に開始されます',
      manualKernelTip: 'カーネルは手動で起動する必要があります',
      enableSuccess: '起動時の自動起動が有効になりました',
      disableSuccess: '起動時の自動起動が無効になりました',
    },
    network: {
      ipv6: 'IPv6優先',
      preferIpv6: 'IPv6接続を優先',
      onlyIpv4: 'IPv4接続のみ使用',
    },
    about: {
      title: '情報',
      appVersion: 'アプリバージョン',
      kernelVersion: 'カーネルバージョン',
      system: 'システム',
      license: 'ライセンス',
      website: 'ウェブサイト',
    },
    error: {
      appDataDir: 'アプリデータディレクトリの取得に失敗しました:',
    },
  },
  rules: {
    title: 'ルール管理',
    add: 'ルール追加',
    type: 'ルールタイプ',
    content: 'ルール内容',
    action: 'アクション',
    noRules: '利用可能なルールがありません',
    targetProxy: 'ターゲットプロキシ',
    directConnect: '直接接続',
    manualSwitch: '手動切替',
    autoSelect: '自動選択',
    logData: 'ルールデータ:',
    fetchSuccess: '{count}個のルールを取得しました',
    fetchError: 'ルールの取得に失敗しました',
    fetchErrorFormat: 'ルールの取得に失敗しました: 無効なデータ形式',
  },
  connections: {
    title: '接続',
    source: 'ソース',
    destination: '宛先',
    duration: '持続時間',
    upload: 'アップロード',
    download: 'ダウンロード',
    noConnections: '接続なし',
    activeConnections: 'アクティブ接続',
    uploadTotal: '総アップロード',
    downloadTotal: '総ダウンロード',
    startTime: '開始時間',
    networkType: 'ネットワーク/タイプ',
    rule: 'ルール',
    ip: 'IP',
    port: 'ポート',
    host: 'ホスト',
  },
  tray: {
    show: 'ウィンドウを表示',
    hide: 'ウィンドウを隠す',
    start: 'サービスを開始',
    stop: 'サービスを停止',
    quit: '終了',
    kernel: 'カーネル',
  },
  notification: {
    kernelStarted: 'カーネルが開始されました',
    kernelStopped: 'カーネルが停止しました',
    kernelRestarted: 'カーネルが再起動されました',
    updateAvailable: '新バージョンが利用可能です',
    updateDownloaded: 'アップデートがダウンロードされました、まもなくインストールされます',
    updateFailed: 'アップデートに失敗しました',
  },
}
