//! 用户自定义规则数据模型。
//!
//! 设计取舍：
//! - 复用 `generic_config` 表的通用 KV 接口（key = `STORAGE_KEY`）持久化，不新建表、不做 schema 迁移。
//! - 自定义规则的生命周期与内核默认规则完全不同：自定义规则持久化到本地、并写入“本程序生成的”
//!   sing-box 配置的 `route.rules`；而内核默认规则的 enable/disable 走 Clash API（运行时、不持久化）。
//!   两者在 UI 与代码里必须明确区分。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// generic_config 中存储自定义规则列表所用的 key。
pub const STORAGE_KEY: &str = "custom_rules";

/// 匹配类型（对应 sing-box route rule 的字段）。新增类型时需同步 `to_route_rule` 实现。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../src/types/generated/CustomRuleMatchType.ts")]
#[serde(rename_all = "snake_case")]
pub enum CustomRuleMatchType {
    /// 精确域名（sing-box `domain`）
    Domain,
    /// 域名后缀（sing-box `domain_suffix`）
    DomainSuffix,
    /// 域名关键字（sing-box `domain_keyword`）
    DomainKeyword,
    /// IP CIDR（sing-box `ip_cidr`）
    IpCidr,
}

impl CustomRuleMatchType {
    /// sing-box route rule 里对应的字段名。
    pub fn singbox_field(&self) -> &'static str {
        match self {
            CustomRuleMatchType::Domain => "domain",
            CustomRuleMatchType::DomainSuffix => "domain_suffix",
            CustomRuleMatchType::DomainKeyword => "domain_keyword",
            CustomRuleMatchType::IpCidr => "ip_cidr",
        }
    }
}

/// 动作：直连 / 走代理 / 拒绝。代理动作可指定具体出站 tag。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../src/types/generated/CustomRuleAction.ts")]
#[serde(rename_all = "snake_case")]
pub enum CustomRuleAction {
    Direct,
    Proxy,
    Block,
}

impl CustomRuleAction {
    /// 解析为 sing-box 的 outbound / action 表达，返回 (字段名, 值)。
    /// - Direct/Proxy → 用 `outbound` 字段（值为出站 tag）
    /// - Block → 用 `action` 字段（值为 `reject`）
    pub fn singbox_outbound_or_action(&self, default_outbound: &str) -> (&'static str, String) {
        match self {
            CustomRuleAction::Direct => (
                "outbound",
                crate::app::singbox::common::TAG_DIRECT.to_string(),
            ),
            CustomRuleAction::Proxy => ("outbound", default_outbound.to_string()),
            CustomRuleAction::Block => ("action", "reject".to_string()),
        }
    }
}

/// 一条自定义规则。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/CustomRule.ts")]
pub struct CustomRule {
    /// 唯一标识（uuid）。前端编辑/删除按此定位。
    pub id: String,
    /// 是否启用。禁用的规则不参与配置生成。
    pub enabled: bool,
    /// 匹配类型
    pub match_type: CustomRuleMatchType,
    /// 匹配内容（域名/IP CIDR/关键字）。多个值用换行或逗号分隔，生成时拆成数组。
    pub payload: String,
    /// 动作
    pub action: CustomRuleAction,
    /// action=proxy 时指定的出站 tag；None 表示用默认出站（自动/手动）。
    /// 当前 MVP 不暴露给用户选择，统一走默认出站，保留字段以便后续扩展。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outbound: Option<String>,
    /// 备注（可选）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// 创建时间
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    /// 更新时间
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

impl CustomRule {
    /// 把该规则转换为 sing-box route rule JSON 对象。
    /// `default_outbound` 是 action=Proxy 时使用的出站 tag。
    /// 返回 None 表示规则无效（如 payload 为空）。
    pub fn to_route_rule(&self, default_outbound: &str) -> Option<Value> {
        if !self.enabled {
            return None;
        }
        let values: Vec<String> = self
            .payload
            .split([',', '\n'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if values.is_empty() {
            return None;
        }

        let (field_key, field_value) = (
            self.match_type.singbox_field(),
            Value::Array(values.into_iter().map(Value::String).collect()),
        );
        let (action_key, action_value) = self.action.singbox_outbound_or_action(default_outbound);

        let mut obj = serde_json::Map::new();
        obj.insert(field_key.to_string(), field_value);
        obj.insert(action_key.to_string(), Value::String(action_value));
        Some(Value::Object(obj))
    }
}

use serde_json::Value;

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(
        match_type: CustomRuleMatchType,
        action: CustomRuleAction,
        payload: &str,
    ) -> CustomRule {
        CustomRule {
            id: "test".to_string(),
            enabled: true,
            match_type,
            payload: payload.to_string(),
            action,
            outbound: None,
            note: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn domain_suffix_direct_emits_outbound_direct() {
        let r = rule(
            CustomRuleMatchType::DomainSuffix,
            CustomRuleAction::Direct,
            "example.com",
        );
        let v = r.to_route_rule("手动切换").unwrap();
        assert_eq!(v["domain_suffix"][0].as_str().unwrap(), "example.com");
        assert_eq!(v["outbound"].as_str().unwrap(), "direct");
    }

    #[test]
    fn ip_cidr_block_emits_action_reject() {
        let r = rule(
            CustomRuleMatchType::IpCidr,
            CustomRuleAction::Block,
            "10.0.0.0/8",
        );
        let v = r.to_route_rule("手动切换").unwrap();
        assert_eq!(v["ip_cidr"][0].as_str().unwrap(), "10.0.0.0/8");
        assert_eq!(v["action"].as_str().unwrap(), "reject");
    }

    #[test]
    fn proxy_uses_default_outbound() {
        let r = rule(
            CustomRuleMatchType::Domain,
            CustomRuleAction::Proxy,
            "openai.com",
        );
        let v = r.to_route_rule("自动选择").unwrap();
        assert_eq!(v["outbound"].as_str().unwrap(), "自动选择");
    }

    #[test]
    fn multiple_payload_values_split_into_array() {
        let r = rule(
            CustomRuleMatchType::DomainSuffix,
            CustomRuleAction::Direct,
            "a.com, b.com\nc.com",
        );
        let v = r.to_route_rule("手动切换").unwrap();
        let arr = v["domain_suffix"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str().unwrap(), "a.com");
        assert_eq!(arr[2].as_str().unwrap(), "c.com");
    }

    #[test]
    fn disabled_rule_returns_none() {
        let mut r = rule(
            CustomRuleMatchType::Domain,
            CustomRuleAction::Direct,
            "x.com",
        );
        r.enabled = false;
        assert!(r.to_route_rule("自动选择").is_none());
    }

    #[test]
    fn empty_payload_returns_none() {
        let r = rule(
            CustomRuleMatchType::Domain,
            CustomRuleAction::Direct,
            "   ,  ",
        );
        assert!(r.to_route_rule("自动选择").is_none());
    }

    #[test]
    fn serde_roundtrip() {
        let r = rule(
            CustomRuleMatchType::DomainKeyword,
            CustomRuleAction::Block,
            "ads",
        );
        let json = serde_json::to_string(&r).unwrap();
        let back: CustomRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.match_type, CustomRuleMatchType::DomainKeyword);
        assert_eq!(back.action, CustomRuleAction::Block);
    }
}
