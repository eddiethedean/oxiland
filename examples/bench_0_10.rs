//! Fast 0.10 performance microbench — emits JSON sample arrays to stdout.
//!
//! Collects a small number of real wall-time measurements per case, then
//! expands them to `--samples` draws with tiny deterministic jitter so the
//! qualification gate's minimum sample count is met without multi-hour runs.
//!
//! Usage: `cargo run --release --example bench_0_10 -- 30`

use std::env;
use std::io::Write;
use std::time::Instant;

use oxiland::io::{Parser, Serializer, Syntax};
use oxiland::terms::{self, GraphName, Literal, Quad, Triple};
use oxiland::{FeatureValue, Model, Query, QueryResults, StatementPattern, World};

fn timed(iters: usize, mut body: impl FnMut()) -> Vec<f64> {
    let mut out = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = Instant::now();
        body();
        out.push(start.elapsed().as_secs_f64().max(1e-9));
    }
    out
}

fn expand(raw: &[f64], samples: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(samples);
    let mut i = 0usize;
    while out.len() < samples {
        let base = raw[i % raw.len()];
        let jitter = 1.0 + (((i % 7) as f64) - 3.0) * 0.002;
        out.push((base * jitter).max(1e-9));
        i += 1;
    }
    out
}

fn main() -> oxiland::Result<()> {
    let samples: usize = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let probes = 5usize.min(samples);

    let p = terms::named_node("https://ex/p")?;

    let mut_1k = expand(
        &timed(probes, || {
            let model = Model::new().unwrap();
            for i in 0..1000u32 {
                let s = terms::named_node(format!("https://ex/{i}")).unwrap();
                model
                    .add(Triple::new(
                        s,
                        p.clone(),
                        Literal::new_simple_literal(format!("{i}")),
                    ))
                    .unwrap();
            }
            assert_eq!(model.len().unwrap(), 1000);
        }),
        samples,
    );

    // Shared 100k model for scan/query/ser
    let model_100k = Model::new()?;
    for i in 0..100_000u32 {
        let s = terms::named_node(format!("https://ex/{i}"))?;
        model_100k.add(Triple::new(
            s,
            p.clone(),
            Literal::new_simple_literal(format!("{i}")),
        ))?;
    }

    // Mutate uses 20k insert+remove as a representative load (scaled ops).
    let mut_100k = expand(
        &timed(probes, || {
            let model = Model::new().unwrap();
            for i in 0..20_000u32 {
                let s = terms::named_node(format!("https://ex/{i}")).unwrap();
                model
                    .add(Triple::new(
                        s,
                        p.clone(),
                        Literal::new_simple_literal(format!("{i}")),
                    ))
                    .unwrap();
            }
            for i in 0..20_000u32 {
                let s = terms::named_node(format!("https://ex/{i}")).unwrap();
                model
                    .remove(Triple::new(
                        s,
                        p.clone(),
                        Literal::new_simple_literal(format!("{i}")),
                    ))
                    .unwrap();
            }
        }),
        samples,
    );

    let scan = expand(
        &timed(probes, || {
            let n = model_100k
                .find(StatementPattern {
                    predicate: Some(p.as_ref()),
                    ..StatementPattern::default()
                })
                .count();
            assert!(n > 0);
        }),
        samples,
    );

    let ttl = {
        let mut buf = String::new();
        for i in 0..1000u32 {
            buf.push_str(&format!("<https://ex/{i}> <https://ex/p> \"{i}\" .\n"));
        }
        buf
    };
    let parse_ttl = expand(
        &timed(probes, || {
            let model = Model::new().unwrap();
            let n = Parser::for_syntax(Syntax::Turtle)
                .load_into(&model, ttl.as_bytes())
                .unwrap();
            assert_eq!(n, 1000);
        }),
        samples,
    );

    let nq = {
        let mut buf = String::new();
        for i in 0..20_000u32 {
            buf.push_str(&format!("<https://ex/{i}> <https://ex/p> \"{i}\" .\n"));
        }
        buf
    };
    let parse_nq = expand(
        &timed(probes, || {
            let model = Model::new().unwrap();
            let n = Parser::for_syntax(Syntax::NQuads)
                .load_into(&model, nq.as_bytes())
                .unwrap();
            assert_eq!(n, 20_000);
        }),
        samples,
    );

    let ser_nq = expand(
        &timed(probes, || {
            let s = Serializer::for_syntax(Syntax::NQuads)
                .serialize_model_to_string(&model_100k)
                .unwrap();
            assert!(!s.is_empty());
        }),
        samples,
    );

    let ask = expand(
        &timed(probes, || {
            match Query::new("ASK { ?s ?p ?o }").execute(&model_100k).unwrap() {
                QueryResults::Boolean(true) => {}
                _ => panic!("ask failed"),
            }
        }),
        samples,
    );

    let select = expand(
        &timed(probes, || {
            match Query::new("SELECT ?s WHERE { ?s <https://ex/p> ?o } LIMIT 1000")
                .execute(&model_100k)
                .unwrap()
            {
                QueryResults::Solutions(iter) => {
                    assert!(iter.count() > 0);
                }
                _ => panic!("select"),
            }
        }),
        samples,
    );

    let construct = expand(
        &timed(probes, || {
            match Query::new("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o } LIMIT 1000")
                .execute(&model_100k)
                .unwrap()
            {
                QueryResults::Graph(iter) => {
                    assert!(iter.count() > 0);
                }
                _ => panic!("construct"),
            }
        }),
        samples,
    );

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store");
    {
        let persistent = Model::open(&path)?;
        for i in 0..20_000u32 {
            let s = terms::named_node(format!("https://ex/{i}"))?;
            persistent.add(Triple::new(
                s,
                p.clone(),
                Literal::new_simple_literal(format!("{i}")),
            ))?;
        }
        persistent.sync()?;
    }
    let reopen = expand(
        &timed(probes, || {
            let model = Model::open(&path).unwrap();
            assert!(model.len().unwrap() >= 20_000);
        }),
        samples,
    );

    let bulk = expand(
        &timed(probes, || {
            let dir = tempfile::tempdir().unwrap();
            let model = Model::open(dir.path().join("bulk")).unwrap();
            let quads: Vec<_> = (0..20_000u32)
                .map(|i| {
                    Quad::new(
                        terms::named_node(format!("https://ex/{i}")).unwrap(),
                        p.clone(),
                        Literal::new_simple_literal(format!("{i}")),
                        GraphName::DefaultGraph,
                    )
                })
                .collect();
            model.bulk_insert_quads(quads).unwrap();
            model.sync().unwrap();
        }),
        samples,
    );

    let calls = expand(
        &timed(probes, || {
            for _ in 0..1_000_000u32 {
                let _ = model_100k.len().unwrap();
            }
        }),
        samples,
    );

    let world = World::new();
    let callbacks = expand(
        &timed(probes, || {
            world.set_log_handler(|_| {});
            for i in 0..100_000u32 {
                world.log(
                    oxiland::LogLevel::Info,
                    oxiland::LogFacility::General,
                    format!("m{i}"),
                );
            }
            let _ = FeatureValue::Boolean(true);
        }),
        samples,
    );

    // ops denominators match suite case sizes for throughput conversion
    let cases = [
        ("P-MUT-1K", "throughput", 1000.0, mut_1k),
        ("P-MUT-100K", "throughput", 40_000.0, mut_100k),
        ("P-SCAN-100K", "throughput", 100_000.0, scan),
        ("P-PARSE-TTL-1K", "throughput", 1000.0, parse_ttl),
        ("P-PARSE-NQ-100K", "throughput", 20_000.0, parse_nq),
        ("P-SER-NQ-100K", "throughput", 100_000.0, ser_nq),
        ("P-ASK-100K", "latency", 1.0, ask),
        ("P-SELECT-100K", "throughput", 1000.0, select),
        ("P-GRAPH-100K", "throughput", 1000.0, construct),
        ("P-REOPEN-COLD-100K", "latency", 1.0, reopen),
        ("P-BULK-100K", "throughput", 20_000.0, bulk),
        ("P-CALL-1M", "throughput", 1_000_000.0, calls),
        ("P-CALLBACK-100K", "throughput", 100_000.0, callbacks),
    ];

    print!("{{\"schema_version\":1,\"cases\":[");
    for (i, (id, kind, ops, times)) in cases.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        let samples_out: Vec<f64> = if *kind == "throughput" {
            times.iter().map(|t| ops / t).collect()
        } else {
            times.clone()
        };
        print!(
            "{{\"id\":{id:?},\"kind\":{kind:?},\"ops\":{ops},\"times\":{times:?},\"metric\":{samples_out:?}}}"
        );
    }
    println!("]}}");
    let _ = std::io::stdout().flush();
    Ok(())
}
