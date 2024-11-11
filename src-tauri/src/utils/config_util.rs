use serde::ser::Error;
use serde::Deserialize;
use serde_json::{Result, Value};
use std::fs;

pub struct ConfigUtil {
    json: Value,
    file_path: String,
}

impl ConfigUtil {
    /// 创建新的 JsonUtil 实例，读取指定文件的 JSON 数据
    pub fn new(file_path: &str) -> Result<Self> {
        let data = fs::read_to_string(file_path).unwrap();
        let json = serde_json::from_str(&data)?;

        Ok(ConfigUtil {
            json,
            file_path: file_path.to_string(),
        })
    }

    /// 获取指定属性并映射到实体
    pub fn get_property_as_entity<T: for<'de> Deserialize<'de>>(
        &self,
        target_keys: &[&str],
    ) -> Result<T> {
        let mut current_value = &self.json;

        for key in target_keys {
            if let Some(val) = current_value.get(*key) {
                current_value = val;
            } else {
                return Err(serde_json::Error::custom(format!("Key not found: {}", key)));
            }
        }

        let entity: T = serde_json::from_value(current_value.clone())?;
        Ok(entity)
    }

    /// 修改指定属性为新的值
    pub fn modify_property(&mut self, target_keys: &[&str], new_value: Value) {
        let mut current_value = &mut self.json;

        for (i, key) in target_keys.iter().enumerate() {
            if i == target_keys.len() - 1 {
                // 最后一个键，直接修改
                if let Some(val) = current_value.get_mut(key) {
                    *val = new_value.clone();
                }
                return;
            }

            // 递归进入子对象
            if let Some(val) = current_value.get_mut(key) {
                current_value = val;
            } else {
                // 如果当前键不存在，返回
                return;
            }
        }
    }

    /// 将修改后的 JSON 写回到文件
    pub fn save(&self) -> Result<()> {
        let updated_data = serde_json::to_string_pretty(&self.json)?;
        fs::write(&self.file_path, updated_data).expect("Failed to write to file");
        Ok(())
    }
}
