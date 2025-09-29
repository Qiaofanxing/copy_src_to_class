use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
use std::collections::HashMap;
use structopt::StructOpt;
use walkdir::WalkDir;
use anyhow::{Result, Context, bail};
use log::{info, error, warn};

#[derive(Debug, StructOpt)]
#[structopt(name = "src_to_class", about = "将Java源文件对应的class文件复制到指定目录")]
struct Opt {
    /// 源代码路径文件夹，包含.java文件
    #[structopt(short, long, parse(from_os_str))]
    source_dir: PathBuf,

    /// 编译后的class文件夹
    #[structopt(short, long, parse(from_os_str))]
    class_dir: PathBuf,

    /// 输出目录
    #[structopt(short, long, parse(from_os_str))]
    output_dir: PathBuf,
}

/// Java类文件版本信息
#[derive(Debug, Clone, PartialEq, Eq)]
struct JavaClassVersion {
    major: u16,
    minor: u16,
}

impl JavaClassVersion {
    /// 返回人类可读的JDK版本字符串
    fn to_jdk_version(&self) -> String {
        match self.major {
            45 => "JDK 1.1".to_string(),
            46 => "JDK 1.2".to_string(),
            47 => "JDK 1.3".to_string(),
            48 => "JDK 1.4".to_string(),
            49 => "JDK 5".to_string(),
            50 => "JDK 6".to_string(),
            51 => "JDK 7".to_string(),
            52 => "JDK 8".to_string(),
            53 => "JDK 9".to_string(),
            54 => "JDK 10".to_string(),
            55 => "JDK 11".to_string(),
            56 => "JDK 12".to_string(),
            57 => "JDK 13".to_string(),
            58 => "JDK 14".to_string(),
            59 => "JDK 15".to_string(),
            60 => "JDK 16".to_string(),
            61 => "JDK 17".to_string(),
            62 => "JDK 18".to_string(),
            63 => "JDK 19".to_string(),
            64 => "JDK 20".to_string(),
            65 => "JDK 21".to_string(),
            _ => format!("未知JDK版本 (major: {})", self.major),
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    
    // 检查路径是否存在
    if !opt.source_dir.exists() {
        bail!("源代码路径不存在: {:?}", opt.source_dir);
    }
    
    if !opt.class_dir.exists() {
        bail!("Class路径不存在: {:?}", opt.class_dir);
    }
    
    // 创建输出目录（如果不存在）
    if !opt.output_dir.exists() {
        fs::create_dir_all(&opt.output_dir)?;
    }
    
    // 收集所有源文件（包括Java和非Java文件）
    let (java_files, non_java_files) = collect_source_files(&opt.source_dir)?;
    info!("找到 {} 个Java源文件，{} 个非Java文件", java_files.len(), non_java_files.len());
    
    // 为每个源文件找到对应的class文件
    let mut failed = false;
    
    // 记录源文件和对应的class文件
    let mut source_to_classes: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    
    for java_file in &java_files {
        let java_rel_path = java_file.strip_prefix(&opt.source_dir)
            .with_context(|| format!("无法获取相对路径: {:?}", java_file))?;
        
        let class_files = find_class_files(&opt.class_dir, java_rel_path)?;
        
        if class_files.is_empty() {
            error!("找不到Java文件对应的class文件: {:?}", java_rel_path);
            failed = true;
            break;
        }
        
        source_to_classes.insert(java_rel_path.to_path_buf(), class_files);
    }
    
    // 如果有任何错误，不复制文件
    if failed {
        bail!("部分Java文件找不到对应的class文件，操作取消");
    }
    
    // 用于记录所有class文件的JDK版本
    let mut jdk_versions: HashMap<String, Vec<PathBuf>> = HashMap::new();
    
    // 首先复制非Java文件
    println!("开始复制非Java文件...");
    let mut copied_non_java_files = 0;
    
    for non_java_file in &non_java_files {
        let rel_path = non_java_file.strip_prefix(&opt.source_dir)
            .with_context(|| format!("无法获取相对路径: {:?}", non_java_file))?;
        
        let target_path = opt.output_dir.join(rel_path);
        
        // 确保目标目录存在
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 获取文件大小
        let file_size = non_java_file.metadata()
            .with_context(|| format!("无法获取文件元数据: {:?}", non_java_file))?.len();
        
        println!("非Java文件：{}，大小：{} 字节", rel_path.to_string_lossy(), file_size);
        
        // 复制文件
        fs::copy(non_java_file, &target_path)
            .with_context(|| format!("复制文件失败: {:?} -> {:?}", non_java_file, target_path))?;
        
        copied_non_java_files += 1;
    }
    
    if copied_non_java_files > 0 {
        println!("----------------------------------------");
    }
    
    // 复制所有class文件到输出目录并检查版本
    println!("开始复制Java文件对应的class文件并检查JDK版本...");
    
    let mut copied_files = 0;
    
    for (java_rel_path, class_files) in &source_to_classes {
        let java_file_name = java_rel_path.to_string_lossy();
        println!("----------------------------------------");
        
        for class_file in class_files {
            let rel_path = class_file.strip_prefix(&opt.class_dir)
                .with_context(|| format!("无法获取相对路径: {:?}", class_file))?;
            
            let target_path = opt.output_dir.join(rel_path);
            
            // 确保目标目录存在
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // 获取文件大小
            let file_size = class_file.metadata()
                .with_context(|| format!("无法获取文件元数据: {:?}", class_file))?.len();
            
            // 检查JDK版本
            let jdk_version = match read_class_file_version(class_file) {
                Ok(version) => {
                    let v = version.to_jdk_version();
                    
                    // 记录版本信息
                    jdk_versions.entry(v.clone())
                        .or_insert_with(Vec::new)
                        .push(class_file.clone());
                    
                    v
                },
                Err(err) => {
                    eprintln!("  警告: 无法读取JDK版本: {}", err);
                    "未知版本".to_string()
                }
            };
            
            // 打印详细信息
            println!("源文件：{}，class文件：{}，大小：{} 字节，JDK版本：{}", 
                java_file_name, 
                rel_path.to_string_lossy(), 
                file_size, 
                jdk_version
            );
            
            // 复制文件
            fs::copy(class_file, &target_path)
                .with_context(|| format!("复制文件失败: {:?} -> {:?}", class_file, target_path))?;
            
            copied_files += 1;
        }
    }
    println!("----------------------------------------");
    
    // 打印汇总信息
    println!("\n--- 汇总信息 ---");
    println!("源文件总数: {}", source_to_classes.len());
    println!("class文件总数: {}", copied_files);
    println!("非Java文件总数: {}", copied_non_java_files);
    println!("复制文件总计: {}", copied_files + copied_non_java_files);
    
    // 检查是否有不同的JDK版本
    if jdk_versions.len() > 1 {
        println!("\n-- 不同JDK版本文件统计 --");
        for (version, files) in &jdk_versions {
            println!("{}: {} 个文件", version, files.len());
        }
        
        warn!("警告: 检测到多个不同的JDK版本!");
    } else if !jdk_versions.is_empty() {
        let version = jdk_versions.keys().next().unwrap();
        println!("所有文件JDK版本: {}", version);
    }
    
    info!("成功复制 {} 个class文件和 {} 个非Java文件到 {:?}", copied_files, copied_non_java_files, opt.output_dir);
    Ok(())
}

/// 收集指定目录下的所有源文件，返回Java文件和非Java文件的列表
fn collect_source_files(source_dir: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut java_files = Vec::new();
    let mut non_java_files = Vec::new();
    
    for entry in WalkDir::new(source_dir) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if path.extension().map_or(false, |ext| ext == "java") {
                java_files.push(path.to_path_buf());
            } else {
                non_java_files.push(path.to_path_buf());
            }
        }
    }
    
    Ok((java_files, non_java_files))
}

/// 查找Java文件对应的所有class文件
fn find_class_files(class_dir: &Path, java_rel_path: &Path) -> Result<Vec<PathBuf>> {
    let mut class_files = Vec::new();
    
    // 将Java路径转换为可能的class路径
    let java_file_name = java_rel_path.file_stem()
        .with_context(|| format!("无法获取文件名: {:?}", java_rel_path))?;
    
    let package_path = java_rel_path.parent().unwrap_or(Path::new(""));
    let class_dir_with_package = class_dir.join(package_path);
    
    // 如果类路径不存在，返回空列表
    if !class_dir_with_package.exists() {
        return Ok(vec![]);
    }
    
    let class_base_name = java_file_name.to_string_lossy();
    
    // 处理内部类的情况（查找所有BaseClass.class, BaseClass$1.class, BaseClass$InnerClass.class等）
    for entry in WalkDir::new(&class_dir_with_package).max_depth(1) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "class") {
            let file_name = path.file_stem()
                .with_context(|| format!("无法获取文件名: {:?}", path))?
                .to_string_lossy();
            
            // 匹配主类或内部类
            if file_name == class_base_name || file_name.starts_with(&format!("{}$", class_base_name)) {
                class_files.push(path.to_path_buf());
            }
        }
    }
    
    Ok(class_files)
}

/// 读取class文件的版本信息
fn read_class_file_version(path: &Path) -> Result<JavaClassVersion> {
    // 打开文件
    let mut file = fs::File::open(path)
        .with_context(|| format!("无法打开class文件: {:?}", path))?;
    
    // 读取前8个字节
    let mut buffer = [0u8; 8];
    file.read_exact(&mut buffer)
        .with_context(|| format!("无法读取class文件头: {:?}", path))?;
    
    // 检查魔数 (0xCAFEBABE)
    if buffer[0] != 0xCA || buffer[1] != 0xFE || buffer[2] != 0xBA || buffer[3] != 0xBE {
        bail!("无效的class文件格式，魔数不匹配: {:?}", path);
    }
    
    // 读取次版本号和主版本号
    let minor = ((buffer[4] as u16) << 8) | (buffer[5] as u16);
    let major = ((buffer[6] as u16) << 8) | (buffer[7] as u16);
    
    Ok(JavaClassVersion { major, minor })
}
