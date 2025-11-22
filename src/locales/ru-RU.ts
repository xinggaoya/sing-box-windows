export default {
  // 通用词汇
  common: {
    appName: 'Sing-Box Windows',
    ok: 'ОК',
    cancel: 'Отмена',
    confirm: 'Подтвердить',
    save: 'Сохранить',
    delete: 'Удалить',
    error: 'Ошибка',
    refresh: 'Обновить',
    unknown: 'Неизвестно',
    search: 'Поиск',
    minimizedToTray: 'Свернуто в трей',
  },

  // 导航菜单
  nav: {
    navigation: 'Навигация',
    home: 'Главная',
    proxy: 'Прокси',
    connections: 'Соединения',
    logs: 'Журналы',
    rules: 'Правила',
    subscription: 'Подписки',
    settings: 'Настройки',
    expand: 'Развернуть',
    collapse: 'Свернуть',
  },

  // 状态相关
  status: {
    connecting: 'Подключение',
    disconnected: 'Отключено',
    running: 'Запущено',
    starting: 'Запуск',
    stopping: 'Остановка',
    stopped: 'Остановлено',
  },

  // 首页
  home: {
    // 基础操作
    start: 'Запуск',
    stop: 'Остановка',
    restart: 'Перезапуск',
    restartAsAdmin: 'Перезапуск от имени администратора',

    // 状态信息
    restartFailed: 'Ошибка перезапуска',
    nodeModeChangeSuccess: 'Смена режима узла выполнена успешно',
    nodeModeChangeFailed: 'Ошибка смены режима узла',
    restartSuccess: 'Ядро успешно перезапущено.',

    // WebSocket状态
    wsStatus: {
      connected: 'Подключено',
      disconnected: 'Отключено',
    },

    // 管理员状态
    adminStatus: {
      admin: 'Администратор',
      normal: 'Обычный пользователь',
    },

    // 代理头部
    proxyHeader: {
      flowMode: 'Режим проксирования трафика',
      nodeMode: 'Режим проксирования узлов',
    },

    // 状态描述
    status: {
      startingDesc: 'Запуск ядра, пожалуйста подождите...',
      stoppingDesc: 'Остановка ядра, пожалуйста подождите...',
      runningDesc: 'Ядро запущено, сервис прокси доступен',
      stoppedDesc: 'Ядро остановлено, сервис прокси недоступен',
      disconnectedDesc: 'Ядро запущено, но соединение нарушено, пожалуйста проверьте конфигурацию',
    },

    // 流量统计
    traffic: {
      title: 'Статистика трафика',
      uploadSpeed: 'Скорость загрузки',
      downloadSpeed: 'Скорость скачивания',
      uploadTotal: 'Общая загрузка',
      downloadTotal: 'Общее скачивание',
      memory: 'Использование памяти',
      connectionsLabel: 'соединений',
    },

  
    // 代理模式
    proxyMode: {
      system: 'Системный прокси',
      tun: 'TUN режим',
      manual: 'Ручной режим',
      systemTip: 'Автоматическая настройка системного прокси',
      tunTip: 'Использовать виртуальную сетевую карту TUN для проксирования всего трафика, требуются права администратора',
      manualTip: 'Требуется ручная настройка системного прокси',
    },

    // 节点模式
    nodeMode: {
      global: 'Глобальный режим',
      rule: 'Режим правил',
      globalTip: 'Весь трафик идет через прокси',
      ruleTip: 'Умная маршрутизация трафика на основе правил',
    },

    // 切换模式
    switchMode: 'Переключить режим',

    tunConfirm: {
      title: 'Требуются права администратора',
      description: 'Для режима TUN нужно перезапустить приложение с правами администратора. Перезапустить сейчас?',
      confirm: 'Перезапустить как администратор',
    },
  },

  // 代理页面
  proxy: {
    title: 'Настройки прокси',
    subtitle: 'Управление прокси-узлами и тестирование задержки',
    currentMode: 'Текущий режим',
    nodes: 'Узлы',
    testNode: 'Тест',
    testing: 'Тестирование...',
    timeout: 'Таймаут',
    testProgress: 'Прогресс теста:',
    switchFailed: 'Ошибка переключения',
    switchErrorMessage: 'Ошибка переключения, пожалуйста проверьте запущен ли Sing-Box',
    testErrorMessage: 'Ошибка теста задержки, узел может быть недоступен или API не отвечает',
    testFailed: 'Ошибка теста задержки',
    nodeTestFailed: 'Ошибка теста задержки узла',
    nodeTestComplete: 'Тест задержки узла завершен',
    groupTestComplete: 'Групповой тест задержки завершен',
    batchTestComplete: 'Пакетный тест задержки завершен',
    loadSuccess: 'Список прокси загружен успешно',
    loadFailed: 'Ошибка получения списка прокси',
    loadFailedCheck: 'Ошибка получения списка прокси, пожалуйста проверьте запущен ли Sing-Box',
    loadingInfo: 'Загрузка информации о прокси...',
    noProxyGroups: 'Нет групп прокси',
    checkConfigOrRefresh: 'Пожалуйста проверьте файл конфигурации или обновите страницу',
    currentLabel: 'Текущий узел:',
    switchSuccess: 'Группа \'{group}\' переключена на \'{proxy}\'.',
    loadMoreNodes: 'Загрузить ещё узлы',
    loadedCount: 'Показано {loaded} из {total}',
    dashboard: {
      groupTotal: 'Группы',
      nodeTotal: 'Узлы',
      expanded: 'Развернуты',
      testing: 'Тестируется',
    },
  },

  // 连接页面
  connections: {
    title: 'Управление соединениями',
    subtitle: 'Управление и мониторинг сетевых соединений',
    source: 'Источник',
    destination: 'Назначение',
    rule: 'Правило',
    traffic: 'Трафик',
    activeConnections: 'Активные соединения',
    refreshConnections: 'Обновить соединения',
    refreshSuccess: 'Список соединений обновлен',
    refreshError: 'Не удалось обновить список подключений: {error}',
    noActiveConnections: 'В настоящее время нет активных сетевых соединений',
    adjustSearchOrFilters: 'Попробуйте изменить условия поиска или фильтры',
    searchPlaceholder: 'Поиск соединений (ID, IP, домен, процесс)',
    networkTypeFilter: 'Тип сети',
    ruleFilter: 'Фильтр правил',
    noMatchingConnections2: 'Нет соответствующих соединений',
    matchedConnections: 'Найденные соединения',
    secondsAgo: '{count}с назад',
    minutesAgo: '{count}м назад',
    hoursAgo: '{count}ч назад',
  },

  // 日志页面
  log: {
    title: 'Просмотр журналов',
    subtitle: 'Мониторинг системных журналов в реальном времени',
    searchLogs: 'Поиск в журналах...',
    clear: 'Очистить журналы',
    copy: 'Копировать журналы',
    export: 'Экспорт журналов',
    records: 'записей',
    noLogs: 'Нет записей в журнале',
    noLogsDesc: 'В настоящее время нет записей в журнале',
    clearedSuccess: 'Журналы очищены',
    copiedSuccess: 'Журналы скопированы в буфер обмена',
    copyFailed: 'Ошибка копирования',
    exportedSuccess: 'Журналы экспортированы',
    clearSearch: 'Очистить поиск',
    filterType: 'Фильтр типов журналов',
    autoScroll: 'Автопрокрутка',
    manualScroll: 'Ручная прокрутка',
    types: {
      info: 'Информация',
      warning: 'Предупреждение',
      error: 'Ошибка',
      success: 'Успешно',
    },
    latestAt: 'Последнее в {time}',
    noSearchResults: 'Нет логов, соответствующих вашему поиску.',
    adjustSearchFilters: 'Попробуйте изменить поиск или фильтры.',
  },

  // 规则页面
  rules: {
    title: 'Управление правилами',
    subtitle: 'Управление правилами маршрутизации прокси',
    add: 'Добавить правило',
    type: 'Тип правила',
    content: 'Содержимое правила',
    targetProxy: 'Целевой прокси',
    directConnect: 'Прямое локальное соединение',
    blockAction: 'Блокировать',
    noRulesData: 'Нет данных о правилах',
    adjustSearchOrFilters: 'Пожалуйста попробуйте изменить условия поиска или фильтры',
    clickRefreshToGetRules: 'Нажмите кнопку обновления для получения данных о правилах',
    getRules: 'Получить правила',
    clearFilters: 'Очистить фильтры',
    searchPlaceholder: 'Поиск содержимого правил, типа, прокси...',
    ruleTypes: 'Типы правил',
    proxyTargets: 'Целевые прокси',
    totalRules: 'Всего правил',
    matchingRules: 'Найденные правила',
    fetchSuccess: 'Успешно получено {count} правил.',
    fetchError: 'Не удалось получить правила: {error}',
  },

  // 订阅页面
  sub: {
    title: 'Управление подписками',
    subtitle: 'Управление конфигурациями подписок и прокси-узлами',
    total: 'Всего',
    active: 'Активно',
    add: 'Добавить подписку',
    edit: 'Редактировать подписку',
    editConfig: 'Редактировать конфигурацию',
    editCurrentConfig: 'Редактировать текущую конфигурацию',
    saveAndApply: 'Сохранить и применить',
    inUse: 'Используется',
    manual: 'Ручное редактирование',
    deleteSuccess: 'Удалено успешно',
    cannotDeleteActive: 'Невозможно удалить используемую подписку',
    name: 'Имя',
    content: 'Содержимое',
    namePlaceholder: 'Пожалуйста введите имя подписки',
    urlPlaceholder: 'Пожалуйста введите URL подписки',
    manualContentPlaceholder: 'Пожалуйста введите содержимое конфигурации (формат JSON)',
    configContentPlaceholder: 'Содержимое файла конфигурации (формат JSON)',
    nameRequired: 'Пожалуйста введите имя подписки',
    urlRequired: 'Пожалуйста введите URL подписки',
    contentRequired: 'Пожалуйста введите содержимое конфигурации',
    invalidUrl: 'Пожалуйста введите действительный URL',
    neverUsed: 'Никогда не использовалось',
    addAndUseSuccess: 'Подписка добавлена и применена успешно',
    updateSuccess: 'Подписка обновлена успешно',
    operationFailed: 'Ошибка операции: ',
    use: 'Использовать',
    useAgain: 'Использовать снова',
    useSuccess: 'Применено успешно',
    useFailed: 'Ошибка применения: ',
    copyLink: 'Копировать ссылку',
    linkCopied: 'Ссылка скопирована в буфер обмена',
    lastUpdate: 'Последнее обновление',
    noSubs: 'Нет подписок',
    noSubscriptionsYet: 'Вы еще не добавили ни одной подписки',
    addFirstSubscription: 'Добавить первую подписку',
    readConfigFailed: 'Ошибка чтения конфигурации: ',
    configSaved: 'Конфигурация сохранена и применена',
    saveConfigFailed: 'Ошибка сохранения конфигурации: ',
    useOriginal: 'Использовать оригинальную подписку',
    useOriginalConfig: 'Использовать оригинальную подписку напрямую (только замена портов)',
    useExtractedNodes: 'Извлечь узлы в локальный шаблон',
    urlSubscription: 'URL подписка',
    manualConfig: 'Ручная конфигурация',
    manualContent: 'Содержимое ручной конфигурации',
  },

  // 设置页面
  setting: {
    title: 'Настройки',
    subtitle: 'Управление конфигурацией приложения и системными настройками',
    appVersion: 'Версия приложения',
    kernelVersion: 'Версия ядра',
    notInstalled: 'Не установлено',
    newVersionFound: 'Найдена новая версия',

    // 通用设置
    general: {
      title: 'Общие настройки',
      description: 'Настройка языка, сети и других параметров приложения',
    },

    // 主题设置
    theme: {
      light: 'Светлая',
      dark: 'Темная',
    },

    // 语言设置
    language: {
      title: 'Язык',
      auto: 'Авто',
      changed: 'Язык изменен',
      changeSuccess: 'Язык успешно изменен',
      description: 'Выберите язык отображения интерфейса приложения',
    },

    // 网络设置
    network: {
      ipv6: 'Поддержка IPv6',
      ipv4Only: 'Только IPv4',
      ipv6Enabled: 'IPv6 включен',
      ipv6Desc: 'Включить или выключить поддержку сетевого протокола IPv6',
      ipVersionChanged: 'Настройки версии IP изменены',
      ports: 'Настройки портов',
      portsDesc: 'Настройка прокси-порта и порта API',
      configure: 'Настроить порты',
      proxyPort: 'Прокси-порт',
      apiPort: 'Порт API',
      portSettings: 'Настройки портов',
      invalidPort: 'Недействительный номер порта, пожалуйста введите порт между 1024-65535',
      portConflict: 'Прокси-порт и порт API не могут быть одинаковыми',
      restartRequired: 'Требуется перезапуск ядра',
      restartDesc: 'После изменения портов требуется перезапуск ядра для вступления в силу',
      portChanged: 'Настройки портов изменены',
      portChangeSuccess: 'Конфигурация портов успешно обновлена',
    },

    // 更新设置
    update: {
      title: 'Настройки обновления',
      description: 'Настройка автоматического обновления и опций проверки версии',
      checkNow: 'Проверить сейчас',
      checkAgain: 'Проверить снова',
      checking: 'Проверка...',
      autoCheck: 'Автоматическая проверка обновлений',
      autoCheckDesc: 'Автоматически проверять обновления приложения при запуске',
      acceptPrerelease: 'Принимать предварительные версии',
      acceptPrereleaseDesc: 'Разрешить получение и установку предварительных версий (тестовых версий)',
      current: 'Текущая версия',
      currentVersion: 'Текущая версия',
      newVersion: 'Новая версия',
      latestVersion: 'Последняя версия',
      latest: 'Последняя версия',
      hasUpdate: 'Есть обновление',
      confirmUpdate: 'Обновить сейчас?',
      later: 'Позже',
      updateNow: 'Обновить сейчас',
      downloading: 'Загрузка обновления',
      installing: 'Установка',
      preparingDownload: 'Подготовка к загрузке...',
      checkError: 'Ошибка проверки обновления',
      updateFailed: 'Ошибка обновления',
      alreadyLatest: 'Уже установлена последняя версия',
      beta: 'Бета',
      releaseNotes: 'Заметки о выпуске',
      updateNotice: 'Уведомление об обновлении',
      skipVersion: 'Пропустить эту версию',
      prereleaseWarningDesc: 'Предварительные версии могут содержать нестабильные функции и потенциальные проблемы, рекомендуется использовать только в тестовых средах.',
      prereleaseConfirm: 'Подтвердить включение предварительных версий',
      prereleaseConfirmDesc: 'После включения вы будете получать уведомления об обновлениях предварительных версий. Эти версии могут быть нестабильными, вы уверены, что хотите продолжить?',
      prereleaseEnabled: 'Предварительные версии включены',
      prereleaseEnabledDesc: 'Вы будете получать уведомления об обновлениях предварительных версий',
      prereleaseDisabled: 'Предварительные версии отключены',
      prereleaseDisabledDesc: 'Вы будете получать только уведомления об обновлениях стабильных версий',
      newVersionFound: 'Найдена новая версия {version}!',
      downloadStarted: 'Загрузка обновления началась в фоновом режиме.',
      downloadError: 'Ошибка загрузки обновления: {error}',
      skipSuccess: 'Эта версия была успешно пропущена.',
      skipError: 'Не удалось пропустить версию: {error}',
    },

    // 自启动设置
    autoStart: {
      app: 'Запуск с системой',
      appDesc: 'Автоматический запуск приложения при запуске системы',
    },

    // 启动设置
    startup: {
      title: 'Настройки запуска',
      description: 'Настройка запуска приложения и опций автозапуска',
      enabled: 'Автозапуск с системой включен',
      disabled: 'Автозапуск с системой отключен',
      enableSuccess: 'Настройка автозапуска с системой выполнена успешно',
      disableSuccess: 'Автозапуск с системой отключен',
    },

    // 内核管理
    kernel: {
      title: 'Управление ядром',
      description: 'Управление версией ядра Sing-Box и загрузкой',
      download: 'Загрузить ядро',
      redownload: 'Повторная загрузка',
      update: 'Обновить ядро',
      checkInstall: 'Проверить установку',
      downloadComplete: 'Загрузка ядра завершена!',
      installSuccess: 'Проверка установки ядра прошла успешно!',
      installFailed: 'Обнаружен недействительный файл ядра',
      preparingDownload: 'Подготовка к загрузке...',
      downloading: 'Загрузка ядра',
      downloadingDescription: 'Файл ядра загружается, пожалуйста подождите...',
      manualDownload: 'Ручная загрузка',
      manualDownloadTitle: 'Инструкция по ручной загрузке',
      manualDownloadGuide: 'Пожалуйста выполните следующие шаги:\n1. Посетите https://github.com/SagerNet/sing-box/releases/latest\n2. Скачайте соответствующую системную версию sing-box\n3. Поместите извлеченный исполняемый файл в следующую директорию:\nWindows: Пользовательская директория/AppData/Local/sing-box-windows/sing-box/sing-box.exe\nLinux: Пользовательская директория/.local/share/sing-box-windows/sing-box/sing-box\n\nПосле завершения нажмите кнопку "Проверить установку" для проверки успешности установки.',
      installPrompt: 'Пожалуйста загрузите и установите ядро перед использованием.',
      downloadFailedMessage: 'Ошибка загрузки ядра: {error}',
    },
    proxyAdvanced: {
      title: 'Расширенные параметры прокси',
      description: 'Настройте список обхода системного прокси и параметры виртуального адаптера TUN',
      systemBypass: 'Домены обхода системного прокси',
      systemBypassPlaceholder: 'Используйте запятые, точки с запятой или переносы строк',
      systemBypassDesc: 'Пример: localhost;127.*;10.*;192.168.*',
      tunTitle: 'Виртуальный адаптер TUN',
      tunAddressInfo: 'Диапазоны 172.19.0.1/30 и fdfe:dcba:9876::1/126 заданы по умолчанию, как в проекте v2rayN.',
      tunMtu: 'MTU',
      tunStack: 'Сетевая стековая модель',
      enableIpv6: 'Включить IPv6-адрес',
      enableIpv6Desc: 'Отключите, если вам нужен только IPv4-трафик.',
      autoRoute: 'Автомаршрутизация',
      strictRoute: 'Строгая маршрутизация',
      save: 'Сохранить расширенные настройки',
      savedTitle: 'Настройки сохранены',
      savedDesc: 'Новые параметры вступят в силу после следующего запуска или перезапуска ядра',
      stackOptions: {
        system: 'System: использовать сетевой стек ОС',
        gvisor: 'gVisor: виртуальный стек gVisor для всего трафика',
        mixed: 'Mixed: System для TCP и gVisor для UDP (по умолчанию)',
      },
      errors: {
        bypassRequired: 'Укажите хотя бы один домен для обхода',
        invalidMtu: 'Значение MTU должно быть в пределах 576–9000',
        invalidStack: 'Выберите допустимый режим стека',
      },
    },

    // 开发者工具
    developer: {
      title: 'Инструменты разработчика',
      description: 'Инструменты отладки разработчика и опции',
      openDevtools: 'Открыть инструменты разработчика',
      openDevtoolsDesc: 'Открыть инструменты разработчика браузера для отладки и проверки frontend кода',
      open: 'Открыть',
      opened: 'Инструменты разработчика открыты',
      warning: 'Инструменты разработчика предназначены в основном для разработки и отладки, нет необходимости включать для обычного использования',
    },

    // 关于
    about: {
      title: 'О программе',
      description: 'Информация о приложении и подробности версии',
      system: 'Система',
      license: 'Лицензия',
    },
  },

  // Меню трея
  tray: {
    kernel: 'Ядро',
    show: 'Показать окно',
    quit: 'Выйти из приложения',
  },

  // Уведомления
  notification: {
    proxyModeChanged: 'Режим прокси изменен',
    proxyModeChangeFailed: 'Ошибка изменения режима прокси',
    updateAvailable: 'Доступна новая версия',
    prereleaseAvailable: 'Доступна новая предварительная версия',
    updateDownloaded: 'Обновление загружено, скоро будет установлено',
    restartAsAdmin: 'Перезапуск от имени администратора',
    restartFailed: 'Ошибка перезапуска',
    goToSettings: 'Перейти к настройкам',
    updatePrompt: 'Доступна новая версия. Перейдите на страницу настроек для обновления.',
  },
}
