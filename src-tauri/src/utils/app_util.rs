

// 获取工作目录
pub fn get_work_dir() -> String {
    let path = std::env::current_exe().unwrap();
    // 仅保留路径 不需要文件
    let path = path.parent().unwrap();
    path.to_str().unwrap().to_string()
}