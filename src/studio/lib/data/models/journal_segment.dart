/// 一段 journal 文本：标注的最小单位。
///
/// 由 journal 文件按空行与 `---` 分隔线切出，一段一屏一决策。
class JournalSegment {
  const JournalSegment({
    required this.id,
    required this.path,
    required this.domain,
    required this.text,
  });

  /// 相对仓库根的来源与段号，如 `data/journal/write/2026-10/2026-10-13.md#2`。
  final String id;

  /// 来源文件的绝对路径。
  final String path;

  /// journal 下的域目录名（default、write、think…）。
  final String domain;

  /// 段落正文。
  final String text;
}
