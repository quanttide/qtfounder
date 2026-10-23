import 'dart:convert';

import 'package:crypto/crypto.dart';

import 'types.dart';

/// UTF-8 文本的 SHA-256 十六进制摘要。
ContentHash hashOf(String content) =>
    sha256.convert(utf8.encode(content)).toString();
