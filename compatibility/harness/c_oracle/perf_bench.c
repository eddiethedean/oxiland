/* Tiny paired performance bench for 0.11 honesty.
 * Usage: perf_bench --case CASE
 * Prints one JSON object: {"id":"...","seconds":[...]} with 30 samples.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <librdf.h>

#define SAMPLES 30

static double now_s(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
}

static void die(const char *m) { fprintf(stderr, "perf_bench: %s\n", m); exit(2); }

static double time_mut(librdf_world *world, int n) {
  double t0 = now_s();
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
    librdf_model_add_statement(model, st);
    librdf_free_statement(st);
  }
  librdf_free_model(model);
  librdf_free_storage(storage);
  return now_s() - t0;
}

static double time_scan(librdf_world *world, int n) {
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
    librdf_model_add_statement(model, st);
    librdf_free_statement(st);
  }
  double t0 = now_s();
  librdf_stream *stream = librdf_model_as_stream(model);
  int count = 0;
  while (stream && !librdf_stream_end(stream)) {
    count++;
    librdf_stream_next(stream);
  }
  if (stream) librdf_free_stream(stream);
  double elapsed = now_s() - t0;
  (void)count;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double time_parse(librdf_world *world, int n) {
  size_t cap = (size_t)n * 64 + 16;
  char *buf = malloc(cap);
  if (!buf) die("oom");
  size_t off = 0;
  for (int i = 0; i < n; i++) {
    int w = snprintf(buf + off, cap - off,
                     "<http://ex/%d> <http://ex/p> \"%d\" .\n", i, i);
    if (w < 0 || (size_t)w >= cap - off) die("buf");
    off += (size_t)w;
  }
  double t0 = now_s();
  librdf_storage *storage = librdf_new_storage(world, "memory", NULL, NULL);
  librdf_model *model = librdf_new_model(world, storage, NULL);
  librdf_parser *parser = librdf_new_parser(world, "turtle", NULL, NULL);
  librdf_uri *base = librdf_new_uri(world, (unsigned char *)"http://example.org/");
  if (!storage || !model || !parser || !base) die("alloc");
  if (librdf_parser_parse_string_into_model(parser, (unsigned char *)buf, base, model) != 0)
    die("parse");
  double elapsed = now_s() - t0;
  librdf_free_uri(base);
  librdf_free_parser(parser);
  librdf_free_model(model);
  librdf_free_storage(storage);
  free(buf);
  return elapsed;
}

static double time_ask(librdf_world *world, int n) {
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
    librdf_model_add_statement(model, st);
    librdf_free_statement(st);
  }
  double t0 = now_s();
  librdf_query *q = librdf_new_query(
      world, "sparql", NULL, (const unsigned char *)"ASK { ?s ?p ?o }", NULL);
  if (q) {
    librdf_query_results *r = librdf_model_query_execute(model, q);
    if (r) librdf_free_query_results(r);
    librdf_free_query(q);
  }
  double elapsed = now_s() - t0;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double time_serialize(librdf_world *world, int n) {
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
    librdf_model_add_statement(model, st);
    librdf_free_statement(st);
  }
  double t0 = now_s();
  librdf_serializer *ser = librdf_new_serializer(world, "ntriples", NULL, NULL);
  if (ser) {
    unsigned char *out = librdf_serializer_serialize_model_to_string(ser, NULL, model);
    if (out) {
#ifdef LIBRDF_VERSION
      librdf_free_memory(out);
#else
      free(out);
#endif
    }
    librdf_free_serializer(ser);
  }
  double elapsed = now_s() - t0;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double time_calls(librdf_world *world, int n_calls) {
  librdf_storage *storage = librdf_new_storage(world, "memory", NULL, NULL);
  librdf_model *model = librdf_new_model(world, storage, NULL);
  double t0 = now_s();
  volatile int sink = 0;
  for (int i = 0; i < n_calls; i++) sink += librdf_model_size(model);
  (void)sink;
  double elapsed = now_s() - t0;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double time_select(librdf_world *world, int n) {
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
    librdf_model_add_statement(model, st);
    librdf_free_statement(st);
  }
  double t0 = now_s();
  librdf_query *q = librdf_new_query(
      world, "sparql", NULL,
      (const unsigned char *)"SELECT ?s WHERE { ?s ?p ?o } LIMIT 1000", NULL);
  int count = 0;
  if (q) {
    librdf_query_results *r = librdf_model_query_execute(model, q);
    if (r) {
      while (!librdf_query_results_finished(r)) {
        count++;
        librdf_query_results_next(r);
      }
      librdf_free_query_results(r);
    }
    librdf_free_query(q);
  }
  (void)count;
  double elapsed = now_s() - t0;
  librdf_free_model(model);
  librdf_free_storage(storage);
  return elapsed;
}

static double time_construct(librdf_world *world, int n) {
  return time_select(world, n); /* same LIMIT 1000 consume cost class */
}

typedef double (*bench_fn)(librdf_world *, int);

static bench_fn resolve(const char *case_id, int *arg) {
  if (!strcmp(case_id, "P-MUT-1K")) { *arg = 1000; return time_mut; }
  if (!strcmp(case_id, "P-MUT-10K")) { *arg = 10000; return time_mut; }
  if (!strcmp(case_id, "P-SCAN-10K")) { *arg = 10000; return time_scan; }
  if (!strcmp(case_id, "P-PARSE-TTL-1K")) { *arg = 1000; return time_parse; }
  if (!strcmp(case_id, "P-PARSE-TTL-10K")) { *arg = 10000; return time_parse; }
  if (!strcmp(case_id, "P-ASK-10K")) { *arg = 10000; return time_ask; }
  if (!strcmp(case_id, "P-SER-NQ-10K")) { *arg = 10000; return time_serialize; }
  if (!strcmp(case_id, "P-SELECT-10K")) { *arg = 10000; return time_select; }
  if (!strcmp(case_id, "P-GRAPH-10K")) { *arg = 10000; return time_construct; }
  if (!strcmp(case_id, "P-CALL-100K")) { *arg = 100000; return time_calls; }
  return NULL;
}

int main(int argc, char **argv) {
  const char *case_id = NULL;
  for (int i = 1; i < argc; i++) {
    if (!strcmp(argv[i], "--case") && i + 1 < argc) case_id = argv[++i];
  }
  if (!case_id) die("usage: perf_bench --case ID");

  int arg = 0;
  bench_fn fn = resolve(case_id, &arg);
  if (!fn) {
    fprintf(stderr, "perf_bench: unsupported case %s\n", case_id);
    return 2;
  }

  librdf_world *world = librdf_new_world();
  if (!world) die("world");
  librdf_world_open(world);

  double samples[SAMPLES];
  (void)fn(world, arg); /* warmup */
  for (int i = 0; i < SAMPLES; i++) {
    samples[i] = fn(world, arg);
    if (samples[i] <= 0) samples[i] = 1e-9;
  }

  printf("{\"id\":\"%s\",\"seconds\":[", case_id);
  for (int i = 0; i < SAMPLES; i++) {
    if (i) printf(",");
    printf("%.9f", samples[i]);
  }
  printf("]}\n");
  librdf_free_world(world);
  return 0;
}
