enum Stratum { none, aiProbability }

class SampleConfig {
  const SampleConfig({
    required this.count,
    this.seed,
    this.includeReviewed = false,
    this.scope,
    this.stratum = Stratum.none,
    this.aiFirst = false,
  });

  final int count;
  final int? seed;
  final bool includeReviewed;
  final Set<String>? scope;
  final Stratum stratum;

  /// 只改变批内顺序，不改变入选概率。
  final bool aiFirst;
}
