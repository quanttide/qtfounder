use std::path::Path;

/// 遍历 Git 仓库 commit，返回每条 commit 的首个 blob 内容（timestamp_secs, content）。
pub fn commit_blobs(repo: &Path, max: usize) -> Vec<(i64, String)> {
    let repo = match git2::Repository::open(repo) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let _ = revwalk.push_head();
    revwalk.set_sorting(git2::Sort::TIME).unwrap_or(());
    let mut results = Vec::new();
    for oid in revwalk.flatten().take(max) {
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let ts = commit.time().seconds();
        for entry in tree.iter().take(1) {
            if let Some(obj) = entry.to_object(&repo).ok() {
                if let Some(blob) = obj.as_blob() {
                    let content = std::str::from_utf8(blob.content())
                        .unwrap_or("")
                        .to_string();
                    results.push((ts, content));
                    break;
                }
            }
        }
    }
    results
}
