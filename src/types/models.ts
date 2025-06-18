// 基本模型类型定义

// 规则类型
export interface Rule {
  type: string
  payload: string
  proxy: string
  size?: number
}
