//! 契约定位：各域共用的一段查找，只差四个参数。
//!
//! 六个域（code／language／docs／meta／agent／write）原本各抄一份，逻辑逐字相同，
//! 差异只有域名字符串、一行注释与换行位置——`code` 那份还少一条相对路径，是抄写漂移
//! 的实样。收敛到这里，「用户级兜底放哪层」这类决策只剩一处。
//!
//! 通用约定：任何第二大脑仓把契约放 `.quanttide/<域>/<文件>`，CLI 进仓即认——
//! 其他领域第二大脑不需要量潮的源码布局，也不需要环境变量。

use std::path::{Path, PathBuf};

use crate::core::path::upwards;

/// 按三层顺序找默认契约：环境变量 → 从当前目录向上逐层试（`.quanttide/<域>/<文件>` 优先）→ 用户级目录。
///
/// - `env`：环境变量名，如 `QTFOUNDER_CODE_CONTRACT`；
/// - `relatives`：源码树里的相对路径，多写几条是为了在本仓开发时也找得到；
/// - `domain`：域名，定通用约定 `.quanttide/<domain>/` 与用户级目录 `~/.config/qtfounder/<domain>/`，也进报错；
/// - `file`：契约文件名。
///
/// 三层都空就报错，**不内置兜底**——契约是显性文件，静默用别处那份等于两份量法。
pub fn find(env: &str, relatives: &[&str], domain: &str, file: &str) -> Result<PathBuf, String> {
    if let Ok(from_env) = std::env::var(env)
        && !from_env.is_empty()
    {
        let path = PathBuf::from(&from_env);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("{env} 指向的契约不存在：{from_env}"));
    }

    let start = std::env::current_dir().map_err(|error| format!("读不到当前目录：{error}"))?;
    let portable = format!(".quanttide/{domain}/{file}");
    for dir in upwards(&start, None) {
        // 通用约定优先：任何第二大脑仓把契约放 .quanttide/<域>/ 下，进仓即认。
        let portable_candidate = dir.join(&portable);
        if portable_candidate.is_file() {
            return Ok(portable_candidate);
        }
        for relative in relatives {
            let candidate = dir.join(relative);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    // 装成命令行工具后，当前目录里不会有源码树的契约——退回用户级目录，
    // 这样 `qtfounder {domain} .` 在任何目录都能跑，用户改阈值也改这一份。
    if let Ok(home) = std::env::var("HOME") {
        let user_level = Path::new(&home)
            .join(".config/qtfounder")
            .join(domain)
            .join(file);
        if user_level.is_file() {
            return Ok(user_level);
        }
    }
    Err(format!(
        "找不到默认契约 {file}：用 --rules 指定，或设 {env}，或放在 .quanttide/{domain}/{file}，或装到 ~/.config/qtfounder/{domain}/{file}"
    ))
}
