//! 极简 protobuf 解码器（仅支持我们需要的子集）
//!
//! 实现 protobuf wire format：
//! - wire type 0 = varint（int32/int64/uint32/uint64/bool/enum）
//! - wire type 1 = 64-bit（fixed64/sfixed64/double）—— 不使用
//! - wire type 2 = length-delimited（string/bytes/embedded message）
//! - wire type 5 = 32-bit（fixed32/sfixed32/float）—— 不使用
//!
//! 参考：<https://protobuf.dev/programming-guides/encoding/>

use super::types::{
    ClashModeStatus, Connection, ConnectionEvent, ConnectionEventType, ConnectionEvents, Group,
    GroupItem, Groups, Log, LogEntry, LogLevel, OutboundList, ServiceStatus, ServiceStatusSnapshot,
    Status,
};
// 1.14 新增类型（RuleList / ServiceList / NetworkQualityResult）在 decoder 内 use，避免循环依赖

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("buffer underflow at offset {0}")]
    Underflow(usize),
    #[error("invalid varint at offset {0}")]
    InvalidVarint(usize),
    #[error("invalid utf-8 at offset {0}")]
    InvalidUtf8(usize),
    #[error("invalid enum value {0} at field {1}")]
    InvalidEnum(i32, u32),
    #[error("unknown wire type {0} at field {1}")]
    UnknownWire(u8, u32),
    #[error("length mismatch: tag at offset {0} expects {1} bytes but only {2} available")]
    LengthMismatch(usize, usize, usize),
}

/// Protobuf 解码器游标
pub struct Decoder<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    pub fn eof(&self) -> bool {
        self.pos >= self.buf.len()
    }

    /// 读取 1 字节
    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        if self.pos >= self.buf.len() {
            return Err(DecodeError::Underflow(self.pos));
        }
        let b = self.buf[self.pos];
        self.pos += 1;
        Ok(b)
    }

    /// 读取 varint（LEB128）
    pub fn read_varint(&mut self) -> Result<u64, DecodeError> {
        let mut result: u64 = 0;
        let mut shift = 0u32;
        let start = self.pos;
        loop {
            if self.pos >= self.buf.len() {
                return Err(DecodeError::InvalidVarint(start));
            }
            let b = self.buf[self.pos];
            self.pos += 1;
            result |= ((b & 0x7f) as u64) << shift;
            if (b & 0x80) == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 64 {
                return Err(DecodeError::InvalidVarint(start));
            }
        }
    }

    /// 读取 wire-type-2 的 length-delimited 字段体，返回 body 切片引用
    pub fn read_bytes(&mut self) -> Result<&'a [u8], DecodeError> {
        let len = self.read_varint()? as usize;
        let start = self.pos;
        if self.pos + len > self.buf.len() {
            return Err(DecodeError::LengthMismatch(start, len, self.remaining()));
        }
        let body = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(body)
    }

    /// 读取 string（wire-type-2 + UTF-8）
    pub fn read_string(&mut self) -> Result<String, DecodeError> {
        let body = self.read_bytes()?;
        let start = self.pos - body.len();
        String::from_utf8(body.to_vec()).map_err(|_| DecodeError::InvalidUtf8(start))
    }

    /// 读取 bool（wire-type-0）
    pub fn read_bool(&mut self) -> Result<bool, DecodeError> {
        Ok(self.read_varint()? != 0)
    }

    /// 读取 i32（wire-type-0，解码为 varint 后转 i32）
    pub fn read_i32(&mut self) -> Result<i32, DecodeError> {
        Ok(self.read_varint()? as i32)
    }

    /// 读取 i64（wire-type-0）
    pub fn read_i64(&mut self) -> Result<i64, DecodeError> {
        Ok(self.read_varint()? as i64)
    }

    /// 跳过当前字段（按 wire type 处理）
    pub fn skip_field(&mut self, wire_type: u8) -> Result<(), DecodeError> {
        match wire_type {
            0 => {
                self.read_varint()?;
            }
            2 => {
                let _ = self.read_bytes()?;
            }
            _ => return Err(DecodeError::UnknownWire(wire_type, 0)),
        }
        Ok(())
    }
}

// ============ Message 解码函数 ============

pub fn decode_groups(buf: &[u8]) -> Result<Groups, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut groups = Groups::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 2) => {
                let body = dec.read_bytes()?;
                groups.group.push(decode_group(body)?);
            }
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(groups)
}

fn decode_group(buf: &[u8]) -> Result<Group, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut group = Group::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 2) => group.tag = dec.read_string()?,
            (2, 2) => group.group_type = dec.read_string()?,
            (3, 0) => group.selectable = dec.read_bool()?,
            (4, 2) => group.selected = dec.read_string()?,
            (5, 0) => group.is_expand = dec.read_bool()?,
            (6, 2) => {
                let body = dec.read_bytes()?;
                group.items.push(decode_group_item(body)?);
            }
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(group)
}

fn decode_group_item(buf: &[u8]) -> Result<GroupItem, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut item = GroupItem::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 2) => item.tag = dec.read_string()?,
            (2, 2) => item.item_type = dec.read_string()?,
            (3, 0) => item.url_test_time = dec.read_i64()?,
            (4, 0) => item.url_test_delay = dec.read_i32()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(item)
}

pub fn decode_outbound_list(buf: &[u8]) -> Result<OutboundList, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut out = OutboundList::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 2) => {
                let body = dec.read_bytes()?;
                out.outbounds.push(decode_group_item(body)?);
            }
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(out)
}

pub fn decode_status(buf: &[u8]) -> Result<Status, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut s = Status::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 0) => s.memory = dec.read_varint()?,
            (2, 0) => s.goroutines = dec.read_i32()?,
            (3, 0) => s.connections_in = dec.read_i32()?,
            (4, 0) => s.connections_out = dec.read_i32()?,
            (5, 0) => s.traffic_available = dec.read_bool()?,
            (6, 0) => s.uplink = dec.read_i64()?,
            (7, 0) => s.downlink = dec.read_i64()?,
            (8, 0) => s.uplink_total = dec.read_i64()?,
            (9, 0) => s.downlink_total = dec.read_i64()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(s)
}

pub fn decode_log(buf: &[u8]) -> Result<Log, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut log = Log::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 2) => {
                let body = dec.read_bytes()?;
                log.messages.push(decode_log_message(body)?);
            }
            (2, 0) => log.reset = dec.read_bool()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(log)
}

fn decode_log_message(buf: &[u8]) -> Result<LogEntry, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut entry = LogEntry::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 0) => {
                let v = dec.read_i32()?;
                entry.level = LogLevel::from(v);
            }
            (2, 2) => entry.message = dec.read_string()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(entry)
}

pub fn decode_connection_events(buf: &[u8]) -> Result<ConnectionEvents, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut events = ConnectionEvents::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 2) => {
                let body = dec.read_bytes()?;
                events.events.push(decode_connection_event(body)?);
            }
            (2, 0) => events.reset = dec.read_bool()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(events)
}

fn decode_connection_event(buf: &[u8]) -> Result<ConnectionEvent, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut event = ConnectionEvent::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 0) => event.event_type = ConnectionEventType::from(dec.read_i32()?),
            (2, 2) => event.id = dec.read_string()?,
            (3, 2) => {
                let body = dec.read_bytes()?;
                event.connection = Some(decode_connection(body)?);
            }
            (4, 0) => event.uplink_delta = dec.read_i64()?,
            (5, 0) => event.downlink_delta = dec.read_i64()?,
            (6, 0) => event.closed_at = dec.read_i64()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(event)
}

fn decode_connection(buf: &[u8]) -> Result<Connection, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut c = Connection::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 2) => c.id = dec.read_string()?,
            (2, 2) => c.inbound = dec.read_string()?,
            (3, 2) => c.inbound_type = dec.read_string()?,
            (4, 0) => c.ip_version = dec.read_i32()?,
            (5, 2) => c.network = dec.read_string()?,
            (6, 2) => c.source = dec.read_string()?,
            (7, 2) => c.destination = dec.read_string()?,
            (8, 2) => c.domain = dec.read_string()?,
            (9, 2) => c.protocol = dec.read_string()?,
            (10, 2) => c.user = dec.read_string()?,
            (11, 2) => c.from_outbound = dec.read_string()?,
            (12, 0) => c.created_at = dec.read_i64()?,
            (13, 0) => c.closed_at = dec.read_i64()?,
            (14, 0) => c.uplink = dec.read_i64()?,
            (15, 0) => c.downlink = dec.read_i64()?,
            (16, 0) => c.uplink_total = dec.read_i64()?,
            (17, 0) => c.downlink_total = dec.read_i64()?,
            (18, 2) => c.rule = dec.read_string()?,
            (19, 2) => c.outbound = dec.read_string()?,
            (20, 2) => c.outbound_type = dec.read_string()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(c)
}

pub fn decode_service_status(buf: &[u8]) -> Result<ServiceStatusSnapshot, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut snap = ServiceStatusSnapshot::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 0) => {
                let v = dec.read_i32()?;
                snap.status = ServiceStatus::from(v);
            }
            (2, 2) => snap.error_message = dec.read_string()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(snap)
}

pub fn decode_clash_mode_status(buf: &[u8]) -> Result<ClashModeStatus, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut s = ClashModeStatus::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 2) => {
                let body = dec.read_bytes()?;
                let s_str = std::str::from_utf8(body)
                    .map_err(|_| DecodeError::InvalidUtf8(dec.pos() - body.len()))?;
                s.mode_list.push(s_str.to_string());
            }
            (2, 2) => s.current_mode = dec.read_string()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(s)
}

// ============ sing-box 1.14 新增 decoder ============

/// 1.14 GetRules 解码（RuleList 消息）
/// proto 字段号（基于 sing-box 1.14 官方 daemon proto 估算）：
///   RuleList.rules (1) -> repeated Rule
///   Rule.rule_type (1, string) / Rule.payload (2, string) / Rule.outbound (3, string)
pub fn decode_rule_list(buf: &[u8]) -> Result<super::types::RuleList, DecodeError> {
    use super::types::Rule;
    let mut dec = Decoder::new(buf);
    let mut list = super::types::RuleList::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        if (field, wire) == (1, 2) {
            let body = dec.read_bytes()?;
            let mut rdec = Decoder::new(body);
            let mut rule = Rule::default();
            while !rdec.eof() {
                let rtag = rdec.read_varint()?;
                let rfield = (rtag >> 3) as u32;
                let rwire = (rtag & 0x07) as u8;
                match (rfield, rwire) {
                    (1, 2) => rule.rule_type = rdec.read_string()?,
                    (2, 2) => rule.payload = rdec.read_string()?,
                    (3, 2) => rule.outbound = rdec.read_string()?,
                    _ => rdec.skip_field(rwire)?,
                }
            }
            list.rules.push(rule);
        } else {
            dec.skip_field(wire)?;
        }
    }
    Ok(list)
}

/// 1.14 GetServices 解码（ServiceList 消息）
/// proto 字段号：
///   ServiceList.services (1) -> repeated Service
///   Service.name (1, string) / Service.type (2, string) / Service.running (3, bool)
pub fn decode_service_list(buf: &[u8]) -> Result<super::types::ServiceList, DecodeError> {
    use super::types::Service;
    let mut dec = Decoder::new(buf);
    let mut list = super::types::ServiceList::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        if (field, wire) == (1, 2) {
            let body = dec.read_bytes()?;
            let mut sdec = Decoder::new(body);
            let mut svc = Service::default();
            while !sdec.eof() {
                let stag = sdec.read_varint()?;
                let sfield = (stag >> 3) as u32;
                let swire = (stag & 0x07) as u8;
                match (sfield, swire) {
                    (1, 2) => svc.name = sdec.read_string()?,
                    (2, 2) => svc.r#type = sdec.read_string()?,
                    (3, 0) => svc.running = sdec.read_bool()?,
                    _ => sdec.skip_field(swire)?,
                }
            }
            list.services.push(svc);
        } else {
            dec.skip_field(wire)?;
        }
    }
    Ok(list)
}

/// 1.14 NetworkQualityTest 解码
/// proto 字段号：
///   tcp_rtt_ms (1) / download_speed_bps (2) / download_latency_ms (3) /
///   upload_speed_bps (4) / upload_latency_ms (5)
pub fn decode_network_quality_result(buf: &[u8]) -> Result<super::types::NetworkQualityResult, DecodeError> {
    let mut dec = Decoder::new(buf);
    let mut r = super::types::NetworkQualityResult::default();
    while !dec.eof() {
        let tag = dec.read_varint()?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x07) as u8;
        match (field, wire) {
            (1, 0) => r.tcp_rtt_ms = dec.read_i32()?,
            (2, 0) => r.download_speed_bps = dec.read_i64()?,
            (3, 0) => r.download_latency_ms = dec.read_i32()?,
            (4, 0) => r.upload_speed_bps = dec.read_i64()?,
            (5, 0) => r.upload_latency_ms = dec.read_i32()?,
            _ => dec.skip_field(wire)?,
        }
    }
    Ok(r)
}

// ============ Message 编码函数（仅用于请求侧） ============

/// 把 string 编码为 length-delimited bytes（用于 outbound request body）
pub fn encode_string(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() + 8);
    encode_varint(out.len() as u64, &mut out);
    out.extend_from_slice(s.as_bytes());
    out
}

/// 把 i64 编码为 varint
pub fn encode_varint_i64(v: i64) -> Vec<u8> {
    let mut out = Vec::new();
    encode_varint(v as u64, &mut out);
    out
}

pub fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// 构造一个 wire-type-2 字段（field number = N）的 tag varint
pub fn encode_field_tag(field_number: u32) -> Vec<u8> {
    let tag = (field_number << 3) | 2;
    let mut out = Vec::new();
    encode_varint(tag as u64, &mut out);
    out
}

/// 空请求 body（用于 google.protobuf.Empty 类型的方法）
pub fn empty_request_body() -> Vec<u8> {
    Vec::new()
}

// 帮助函数：把 i32 写到 Vec
pub fn encode_i32(v: i32) -> Vec<u8> {
    let mut out = Vec::new();
    encode_varint(v as u64, &mut out);
    out
}

// 让 from_str 在解码错误时方便

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_roundtrip() {
        for v in [0u64, 1, 127, 128, 255, 16384, u32::MAX as u64, u64::MAX] {
            let mut buf = Vec::new();
            encode_varint(v, &mut buf);
            let mut dec = Decoder::new(&buf);
            assert_eq!(dec.read_varint().unwrap(), v);
        }
    }

    #[test]
    fn decode_empty_groups() {
        let groups = decode_groups(&[]).unwrap();
        assert!(groups.group.is_empty());
    }
}

/// 把 `DecodeError` 转换为可跨线程的 `String`
pub fn decode_error_to_string(e: &DecodeError) -> String {
    format!("{}", e)
}