/* Strict paired-sampling driver for the 0.13 faster-than-Redland gate.
 * Reuses the validated workloads from perf_bench.c but allows the controller
 * to request one calibrated sample at a time for true Oxiland/Redland AB/BA
 * interleaving. Reported seconds are normalized per completed workload.
 */
#define main perf_bench_legacy_main
#include "perf_bench.c"
#undef main

static double time_serialize_nquads(librdf_world *world, int n) {
  librdf_storage *storage = librdf_new_storage(world, "memory", NULL, NULL);
  librdf_model *model = librdf_new_model(world, storage, NULL);
  char uri[128];
  for (int i = 0; i < n; i++) {
    snprintf(uri, sizeof(uri), "http://ex/%d", i);
    librdf_node *s = librdf_new_node_from_uri_string(world, (unsigned char *)uri);
    librdf_node *p = librdf_new_node_from_uri_string(world, (unsigned char *)"http://ex/p");
    snprintf(uri, sizeof(uri), "%d", i);
    librdf_node *o = librdf_new_node_from_literal(world, (unsigned char *)uri, NULL, 0);
    librdf_statement *st = librdf_new_statement_from_nodes(world, s, p, o);
    if (!st || librdf_model_add_statement(model, st) != 0) die("serialize setup");
    librdf_free_statement(st);
  }
  commit_setup(model, n, "serialize setup commit");
  double t0 = now_s();
  librdf_serializer *ser = librdf_new_serializer(world, "nquads", NULL, NULL);
  if (!ser) die("serializer");
  unsigned char *out = librdf_serializer_serialize_model_to_string(ser, NULL, model);
  if (out && strlen((const char *)out) > 0) {
#ifdef LIBRDF_VERSION
    librdf_free_memory(out);
#else
    free(out);
#endif
  } else die("serialize validation");
  librdf_free_serializer(ser);
  double elapsed = now_s() - t0;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static bench_fn resolve_0_13(const char *case_id, int *arg) {
  if (!strcmp(case_id, "P-SER-NQ-10K")) {
    *arg = 10000;
    return time_serialize_nquads;
  }
  return resolve(case_id, arg);
}

static void setup_model_0_13(
    librdf_world *world, int n, const char *label,
    librdf_storage **storage_out, librdf_model **model_out) {
  librdf_storage *storage = librdf_new_storage(world, "memory", NULL, NULL);
  librdf_model *model = librdf_new_model(world, storage, NULL);
  if (!storage || !model) die("alloc");
  char uri[128];
  for (int i = 0; i < n; i++) {
    snprintf(uri, sizeof(uri), "http://ex/%d", i);
    librdf_node *s = librdf_new_node_from_uri_string(world, (unsigned char *)uri);
    librdf_node *p = librdf_new_node_from_uri_string(world, (unsigned char *)"http://ex/p");
    snprintf(uri, sizeof(uri), "%d", i);
    librdf_node *o = librdf_new_node_from_literal(world, (unsigned char *)uri, NULL, 0);
    librdf_statement *st = librdf_new_statement_from_nodes(world, s, p, o);
    if (!st || librdf_model_add_statement(model, st) != 0) die(label);
    librdf_free_statement(st);
  }
  commit_setup(model, n, label);
  *storage_out = storage;
  *model_out = model;
}

static double strict_scan(librdf_world *world, int n, int repetitions) {
  librdf_storage *storage;
  librdf_model *model;
  setup_model_0_13(world, n, "scan setup", &storage, &model);
  double t0 = now_s();
  for (int r = 0; r < repetitions; r++) {
    librdf_stream *stream = librdf_model_as_stream(model);
    int count = 0;
    while (stream && !librdf_stream_end(stream)) {
      count++;
      librdf_stream_next(stream);
    }
    if (!stream || count != n) die("scan validation");
    librdf_free_stream(stream);
  }
  double elapsed = (now_s() - t0) / (double)repetitions;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double strict_serialize(librdf_world *world, int n, int repetitions) {
  librdf_storage *storage;
  librdf_model *model;
  setup_model_0_13(world, n, "serialize setup", &storage, &model);
  double t0 = now_s();
  for (int r = 0; r < repetitions; r++) {
    librdf_serializer *ser = librdf_new_serializer(world, "nquads", NULL, NULL);
    unsigned char *out = ser ? librdf_serializer_serialize_model_to_string(ser, NULL, model) : NULL;
    if (!out || strlen((const char *)out) == 0) die("serialize validation");
#ifdef LIBRDF_VERSION
    librdf_free_memory(out);
#else
    free(out);
#endif
    librdf_free_serializer(ser);
  }
  double elapsed = (now_s() - t0) / (double)repetitions;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double strict_ask(librdf_world *world, int n, int repetitions) {
  librdf_storage *storage;
  librdf_model *model;
  setup_model_0_13(world, n, "ask setup", &storage, &model);
  double t0 = now_s();
  for (int r = 0; r < repetitions; r++) {
    librdf_query *q = librdf_new_query(
        world, "sparql", NULL, (const unsigned char *)"ASK { ?s ?p ?o }", NULL);
    librdf_query_results *results = q ? librdf_model_query_execute(model, q) : NULL;
    if (!results || !librdf_query_results_is_boolean(results) ||
        librdf_query_results_get_boolean(results) != 1) die("ask validation");
    librdf_free_query_results(results);
    librdf_free_query(q);
  }
  double elapsed = (now_s() - t0) / (double)repetitions;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double strict_select(librdf_world *world, int n, int repetitions) {
  librdf_storage *storage;
  librdf_model *model;
  setup_model_0_13(world, n, "select setup", &storage, &model);
  double t0 = now_s();
  for (int r = 0; r < repetitions; r++) {
    librdf_query *q = librdf_new_query(
        world, "sparql", NULL,
        (const unsigned char *)"SELECT ?s WHERE { ?s ?p ?o } LIMIT 1000", NULL);
    librdf_query_results *results = q ? librdf_model_query_execute(model, q) : NULL;
    int count = 0;
    if (!results || !librdf_query_results_is_bindings(results)) die("select results");
    while (!librdf_query_results_finished(results)) {
      count++;
      librdf_query_results_next(results);
    }
    if (count != 1000) die("select validation");
    librdf_free_query_results(results);
    librdf_free_query(q);
  }
  double elapsed = (now_s() - t0) / (double)repetitions;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double strict_construct(librdf_world *world, int n, int repetitions) {
  librdf_storage *storage;
  librdf_model *model;
  setup_model_0_13(world, n, "construct setup", &storage, &model);
  double t0 = now_s();
  for (int r = 0; r < repetitions; r++) {
    librdf_query *q = librdf_new_query(
        world, "sparql", NULL,
        (const unsigned char *)"CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o } LIMIT 1000", NULL);
    librdf_query_results *results = q ? librdf_model_query_execute(model, q) : NULL;
    librdf_stream *stream = results ? librdf_query_results_as_stream(results) : NULL;
    int count = 0;
    while (stream && !librdf_stream_end(stream)) {
      count++;
      librdf_stream_next(stream);
    }
    if (!results || !librdf_query_results_is_graph(results) || !stream || count != 1000)
      die("construct validation");
    librdf_free_stream(stream);
    librdf_free_query_results(results);
    librdf_free_query(q);
  }
  double elapsed = (now_s() - t0) / (double)repetitions;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double strict_sample(
    const char *case_id, bench_fn fn, librdf_world *world, int arg, int repetitions) {
  if (!strcmp(case_id, "P-SCAN-10K")) return strict_scan(world, arg, repetitions);
  if (!strcmp(case_id, "P-SER-NQ-10K")) return strict_serialize(world, arg, repetitions);
  if (!strcmp(case_id, "P-ASK-10K")) return strict_ask(world, arg, repetitions);
  if (!strcmp(case_id, "P-SELECT-10K")) return strict_select(world, arg, repetitions);
  if (!strcmp(case_id, "P-GRAPH-10K")) return strict_construct(world, arg, repetitions);
  double total = 0.0;
  for (int i = 0; i < repetitions; i++) total += fn(world, arg);
  return total / (double)repetitions;
}

int main(int argc, char **argv) {
  const char *case_id = NULL;
  int samples_count = 1;
  double target_ms = 10.0;
  for (int i = 1; i < argc; i++) {
    if (!strcmp(argv[i], "--case") && i + 1 < argc) case_id = argv[++i];
    else if (!strcmp(argv[i], "--samples") && i + 1 < argc) samples_count = atoi(argv[++i]);
    else if (!strcmp(argv[i], "--target-ms") && i + 1 < argc) target_ms = strtod(argv[++i], NULL);
  }
  if (!case_id || samples_count < 1 || samples_count > 1000 || target_ms <= 0.0)
    die("usage: perf_bench_0_13 --case ID [--samples N] [--target-ms MS]");

  int arg = 0;
  bench_fn fn = resolve_0_13(case_id, &arg);
  if (!fn) die("unsupported case");

  librdf_world *world = librdf_new_world();
  if (!world) die("world");
  librdf_world_open(world);

  (void)fn(world, arg); /* recorded-process warm-up */
  double calibration = fn(world, arg);
  int repetitions = 1;
  const double target_s = target_ms / 1000.0;
  if (calibration > 0.0 && calibration < target_s) {
    repetitions = (int)(target_s / calibration) + 1;
    if (repetitions > 100000) repetitions = 100000;
  }

  double *samples = calloc((size_t)samples_count, sizeof(*samples));
  if (!samples) die("oom");
  for (int i = 0; i < samples_count; i++) {
    samples[i] = strict_sample(case_id, fn, world, arg, repetitions);
    if (samples[i] <= 0.0) samples[i] = 1e-9;
  }

  printf("{\"id\":\"%s\",\"repetitions\":%d,\"seconds\":[", case_id, repetitions);
  for (int i = 0; i < samples_count; i++) {
    if (i) printf(",");
    printf("%.9f", samples[i]);
  }
  printf("]}\n");
  free(samples);
  librdf_free_world(world);
  return 0;
}
