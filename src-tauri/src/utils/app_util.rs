

// 获取工作目录
pub fn get_work_dir() -> String {
    let path = std::env::current_dir().unwrap();
    path.display().to_string()
}