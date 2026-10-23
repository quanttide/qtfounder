import 'package:path/path.dart' as p;
import 'package:review_tool/review_tool.dart';

/// 创建审阅服务。调用方拿到后先 `await service.init()`。
///
/// 记录默认落在知识库根目录的 `.review/`；需要隔离时显式传
/// [storageDirectory]。Web 端会拿到 unsupported 的 source/store，调用具体
/// 方法时抛 `ReviewUnsupportedError`，入口需要自行判断是否展示审阅功能。
ReviewService createReviewService({
  required String root,
  String? storageDirectory,
}) {
  return ReviewService(
    root: root,
    source: const FsArticleSource(),
    store: JsonlReviewStore(
      directory: storageDirectory ?? p.join(root, '.review'),
    ),
  );
}
