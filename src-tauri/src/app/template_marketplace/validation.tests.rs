use super::{validate_template_content, validate_template_meta, MAX_TEMPLATE_CONTENT_BYTES};

const VALID_MINIMAL: &str = r#"{
    "outbounds": [
        { "type": "selector", "tag": "manual", "outbounds": ["{{NODES}}"] }
    ]
}"#;

#[test]
fn accepts_minimal_valid_template() {
    assert!(validate_template_content(VALID_MINIMAL).is_ok());
}

#[test]
fn rejects_invalid_json() {
    assert!(validate_template_content("{ not json").is_err());
}

#[test]
fn rejects_non_object_root() {
    assert!(validate_template_content("[1,2,3]").is_err());
}

#[test]
fn rejects_missing_outbounds() {
    assert!(validate_template_content(r#"{"log": {"level": "info"}}"#).is_err());
}

#[test]
fn rejects_empty_outbounds() {
    assert!(validate_template_content(r#"{"outbounds": []}"#).is_err());
}

#[test]
fn rejects_oversized_content() {
    let big = format!(
        r#"{{"outbounds":[{{}}],"pad":"{}"}}"#,
        "x".repeat(MAX_TEMPLATE_CONTENT_BYTES)
    );
    assert!(validate_template_content(&big)
        .unwrap_err()
        .contains("过大"));
}

#[test]
fn meta_validation_accepts_normal_input() {
    assert!(validate_template_meta("我的模板", "描述", "anon", "singbox").is_ok());
    // 内核类型大小写归一后接受
    assert!(validate_template_meta("t", "", "", " SingBox ").is_ok());
}

#[test]
fn meta_validation_rejects_blank_name_and_bad_kernel() {
    assert!(validate_template_meta("   ", "", "", "singbox").is_err());
    assert!(validate_template_meta("t", "", "", "clash").is_err());
}
