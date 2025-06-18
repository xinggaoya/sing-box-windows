use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

/// 任务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Running,
    Stopped,
    Failed(String),
}

/// 任务管理器
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, TaskStatus>>>,
    handles: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 启动一个新任务
    pub async fn spawn<F>(&self, id: impl Into<String>, task: F) -> Result<(), String>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let id = id.into();

        // 检查任务是否已存在
        {
            let tasks = self.tasks.read().await;
            if let Some(status) = tasks.get(&id) {
                if *status == TaskStatus::Running {
                    return Err(format!("任务 {} 已在运行中", id));
                }
            }
        }

        // 更新任务状态
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(id.clone(), TaskStatus::Running);
        }

        // 启动任务
        let handle = tokio::spawn(task);

        // 保存任务句柄
        {
            let mut handles = self.handles.lock().await;
            handles.insert(id, handle);
        }

        Ok(())
    }

    /// 停止指定任务
    pub async fn stop(&self, id: &str) -> Result<(), String> {
        let handle = {
            let mut handles = self.handles.lock().await;
            handles.remove(id)
        };

        if let Some(handle) = handle {
            handle.abort();

            let mut tasks = self.tasks.write().await;
            tasks.insert(id.to_string(), TaskStatus::Stopped);

            Ok(())
        } else {
            Err(format!("任务 {} 不存在", id))
        }
    }

    /// 停止所有任务
    pub async fn stop_all(&self) {
        let handles: Vec<_> = {
            let mut handles = self.handles.lock().await;
            handles.drain().map(|(_, h)| h).collect()
        };

        for handle in handles {
            handle.abort();
        }

        let mut tasks = self.tasks.write().await;
        tasks.clear();
    }
}
