class ReviewPathError implements Exception {
  final String message;

  const ReviewPathError(this.message);

  @override
  String toString() => 'ReviewPathError: $message';
}

class ReviewUnsupportedError implements Exception {
  const ReviewUnsupportedError();

  @override
  String toString() => '审阅功能在当前平台不可用';
}

class ReviewValidationError implements Exception {
  final String message;

  const ReviewValidationError(this.message);

  @override
  String toString() => 'ReviewValidationError: $message';
}
