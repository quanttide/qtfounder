import 'dart:math';
import 'dart:typed_data';

import 'interfaces.dart';

/// 行级 diff。超过 [maxLines] 直接返回 null，避免 O(n*m) 把 UI 卡死。
///
/// 输出是展示用格式，不是可 apply 的 unified patch：
/// 每个变更块以 `---` 开头，删除行前缀 `- `，新增行前缀 `+ `。
class LineDiffService implements DiffService {
  const LineDiffService({this.maxLines = 2000});

  final int maxLines;

  @override
  Future<String?> diff(String before, String after) async {
    if (before == after) return null;

    final a = before.isEmpty ? <String>[] : before.split('\n');
    final b = after.isEmpty ? <String>[] : after.split('\n');
    if (a.length > maxLines || b.length > maxLines) return null;

    return _diffLines(a, b);
  }

  String? _diffLines(List<String> a, List<String> b) {
    final n = a.length;
    final m = b.length;

    if (n == 0 && m == 0) return null;
    if (n == 0) return b.map((line) => '+ $line').join('\n');
    if (m == 0) return a.map((line) => '- $line').join('\n');

    final dp = List<Uint32List>.generate(
      n + 1,
      (_) => Uint32List(m + 1),
      growable: false,
    );
    for (var i = 1; i <= n; i++) {
      for (var j = 1; j <= m; j++) {
        dp[i][j] = a[i - 1] == b[j - 1]
            ? dp[i - 1][j - 1] + 1
            : max(dp[i - 1][j], dp[i][j - 1]);
      }
    }

    final ops = <_DiffOp>[];
    var i = n;
    var j = m;
    while (i > 0 || j > 0) {
      if (i > 0 && j > 0 && a[i - 1] == b[j - 1]) {
        ops.add(_DiffOp.equal(a[i - 1]));
        i--;
        j--;
      } else if (j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j])) {
        ops.add(_DiffOp.add(b[j - 1]));
        j--;
      } else {
        ops.add(_DiffOp.remove(a[i - 1]));
        i--;
      }
    }

    final out = StringBuffer();
    var inHunk = false;
    for (final op in ops.reversed) {
      if (op.kind == _DiffKind.equal) {
        inHunk = false;
        continue;
      }
      if (!inHunk) {
        if (out.isNotEmpty) out.writeln();
        out.writeln('---');
        inHunk = true;
      }
      out.writeln('${op.kind == _DiffKind.remove ? '-' : '+'} ${op.line}');
    }

    final result = out.toString().trimRight();
    return result.isEmpty ? null : result;
  }
}

enum _DiffKind { equal, remove, add }

class _DiffOp {
  const _DiffOp.equal(this.line) : kind = _DiffKind.equal;
  const _DiffOp.remove(this.line) : kind = _DiffKind.remove;
  const _DiffOp.add(this.line) : kind = _DiffKind.add;

  final _DiffKind kind;
  final String line;
}
