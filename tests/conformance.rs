use std::fs;
use std::path::PathBuf;

use oxiland::io::{GraphTarget, Parser, Syntax};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Suite {
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    syntax: String,
    input: String,
    expect: String,
    #[serde(default)]
    quads: Option<usize>,
    #[serde(default)]
    base_iri: Option<String>,
    #[serde(default)]
    dataset: bool,
}

#[test]
fn curated_w3c_style_cases_run_through_public_facade() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let suite: Suite = serde_json::from_str(
        &fs::read_to_string(root.join("compatibility/conformance/cases.json")).unwrap(),
    )
    .unwrap();

    for case in suite.cases {
        let data = fs::read(root.join(&case.input)).unwrap();
        let syntax = Syntax::from_name(&case.syntax).unwrap();
        let mut parser = Parser::for_syntax(syntax);
        if let Some(base) = &case.base_iri {
            parser = parser.base_iri(base).unwrap();
        }
        if case.dataset {
            parser = parser.graph_target(GraphTarget::Dataset);
        }
        let parsed = parser
            .parse_slice(&data)
            .and_then(|stream| stream.collect::<Result<Vec<_>, _>>());

        match case.expect.as_str() {
            "pass" => {
                let quads = parsed.unwrap_or_else(|error| {
                    panic!("case {} expected pass, got error: {error}", case.id)
                });
                if let Some(expected) = case.quads {
                    assert_eq!(quads.len(), expected, "case {} quad count", case.id);
                }
            }
            "fail" => {
                assert!(parsed.is_err(), "case {} expected failure", case.id);
            }
            other => panic!("unknown expect {other}"),
        }
    }
}
