import {
  ALL_COUNTRY_CODES,
  COUNTRY_CODE_ALIASES,
  COUNTRY_DISPLAY_LOCALES,
  COUNTRY_NAME_PATTERNS,
  INFORMATIONAL_NODE_PATTERNS,
  PROXY_PREFIX_COUNTRY_CODES,
} from '@/constants/countries'

/**
 * 节点名 → 国家码 → 国旗图标的解析
 *
 * 机场订阅的节点名格式极不统一：`🇭🇰 香港 01`、`HK-01`、`HongKong 1`、
 * `US | Los Angeles Premium`、`CF-日本 IEPL`……要显示国旗就得先把这些
 * 名字归一化成国家码。
 *
 * 解析分三层，从「确定性高」到「确定性低」依次尝试：
 * 1. 地名正则表（东京 / 香港 / 美国 / …）
 * 2. 本地化国名反查（`Intl.DisplayNames` 在 6 种语言下的国名）
 * 3. 裸两字母 token（`HK-01`）→ 直接取，并跳过协议前缀（CF / CL / SS）
 *
 * 注意：节点名前缀的国旗 emoji 只做**剥离**（`stripLeadingFlagEmoji`），
 * 不反解成国家码 —— 后面的三层同样能覆盖这些名字，反解只会多一条需要维护的分支。
 *
 * 国旗资源在构建期由 `import.meta.glob` 收集，运行期只做查表 —— 打进 bundle
 * 的是 URL 字符串而不是 SVG 内容。
 */

const flagIconUrls = import.meta.glob('../../node_modules/flag-icons/flags/4x3/*.svg', {
  eager: true,
  query: '?url',
  import: 'default',
}) as Record<string, string>

/** 国家码 → `flag-icons` 资源 URL 的索引（懒构建，避免启动时遍历整个 map） */
let flagIconIndex: Map<string, string> | null = null

function buildFlagIconIndex(): Map<string, string> {
  if (flagIconIndex) return flagIconIndex

  const index = new Map<string, string>()
  for (const [path, url] of Object.entries(flagIconUrls)) {
    const code = path.match(/\/([A-Z0-9-]+)\.svg$/i)?.[1]?.toUpperCase()
    if (code) index.set(code, url)
  }
  flagIconIndex = index
  return index
}

/** 国家码 → `flag-icons` 用的文件名（小写）。非法码返回空串。 */
export function getFlagIconCountryCode(code: string): string {
  if (code === 'EU') return 'eu'
  const normalized = COUNTRY_CODE_ALIASES[code] || code
  if (!/^[A-Z]{2}$/.test(normalized)) return ''
  return normalized.toLowerCase()
}

/** 去掉节点名前缀的国旗 emoji */
export function stripLeadingFlagEmoji(nodeName: string): string {
  return nodeName.replace(/^[\u{1f1e6}-\u{1f1ff}]{2}\s*/u, '').trim()
}

/** 是否是「流量 / 到期 / 官网」这类信息条目 —— 它们没有国旗，也不该参与测速 */
export function isInformationalNodeName(nodeName: string): boolean {
  return INFORMATIONAL_NODE_PATTERNS.some((pattern) =>
    pattern.test(stripLeadingFlagEmoji(nodeName)),
  )
}

/** 国名归一化：去掉空白与标点，用于「节点名包含国名」的子串匹配 */
export function normalizeCountryLabel(value: string): string {
  return value
    .toLowerCase()
    .replace(/[\s·・,，.。()（）[\]【】'’"“”_-]/g, '')
    .trim()
}

/**
 * 预计算「本地化国名 → 国家码」反查表。
 *
 * 用 `Intl.DisplayNames` 在多种语言下取国名，避免把中文/日文/俄文国名
 * 硬编码成一张表（那样每次内核新增国家都要手工补）。
 * 按标签长度降序排列，保证「先匹配到更具体的名字」。
 */
function getLocalizedCountryNameEntries(): Array<{ code: string; label: string }> {
  const entries: Array<{ code: string; label: string }> = []
  const seen = new Set<string>()

  for (const displayLocale of COUNTRY_DISPLAY_LOCALES) {
    const displayNames = new Intl.DisplayNames([displayLocale], { type: 'region' })
    for (const countryCode of ALL_COUNTRY_CODES) {
      let label: string | undefined
      try {
        label = displayNames.of(countryCode)
      } catch {
        // 区域码（如 ASEAN）在部分 locale 下没有对应名称
        continue
      }
      if (!label || label === countryCode) continue

      const normalizedLabel = normalizeCountryLabel(label)
      // 单字国名（如「中」）子串匹配误报率极高，直接排除
      if (!normalizedLabel || normalizedLabel.length < 2) continue

      const key = `${countryCode}:${normalizedLabel}`
      if (seen.has(key)) continue
      seen.add(key)
      entries.push({ code: countryCode, label: normalizedLabel })
    }
  }

  return entries.sort((left, right) => right.label.length - left.label.length)
}

const localizedCountryNameEntries = getLocalizedCountryNameEntries()

/**
 * 从节点名推断国家码（`flag-icons` 文件名形式）。
 *
 * 无法识别时返回空串 —— 调用方据此不渲染国旗，而不是渲染一个错的。
 */
export function getNodeCountryCode(nodeName: string): string {
  const normalizedName = stripLeadingFlagEmoji(nodeName)
  if (isInformationalNodeName(normalizedName)) return ''

  // 1) 地名正则表
  const override = COUNTRY_NAME_PATTERNS.find(([pattern]) => pattern.test(normalizedName))
  if (override) return getFlagIconCountryCode(override[1])

  // 2) 本地化国名反查
  const normalizedNodeName = normalizeCountryLabel(normalizedName)
  const localizedCountry = localizedCountryNameEntries.find((entry) =>
    normalizedNodeName.includes(entry.label),
  )
  if (localizedCountry) return getFlagIconCountryCode(localizedCountry.code)

  // 3) 裸两字母 token；首个若是协议前缀（CF / CL / SS）则取第二个
  const tokens = normalizedName
    .toUpperCase()
    .split(/[^A-Z]+/)
    .filter(Boolean)
  const countryCodes = tokens.filter((token) => token === 'EU' || /^[A-Z]{2}$/.test(token))
  if (!countryCodes.length) return ''

  const preferredCode =
    countryCodes.length > 1 && PROXY_PREFIX_COUNTRY_CODES.has(countryCodes[0])
      ? countryCodes[1]
      : countryCodes[0]
  return getFlagIconCountryCode(preferredCode)
}

/** 展示用的节点名（去掉前缀国旗 emoji） */
export function getDisplayNodeName(nodeName?: string | null): string {
  return nodeName ? stripLeadingFlagEmoji(nodeName) : '-'
}

/** 节点对应的国旗 URL；无法识别国家时返回空串 */
export function getNodeFlagUrl(nodeName?: string | null): string {
  if (!nodeName) return ''
  const countryCode = getNodeCountryCode(nodeName)
  if (!countryCode) return ''
  return buildFlagIconIndex().get(countryCode.toUpperCase()) ?? ''
}
