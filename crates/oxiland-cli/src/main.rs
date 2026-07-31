//! Command-line workflows for local Oxiland RDF datasets (ADR-019).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use oxigraph::model::{GraphName, NamedOrBlankNode, Quad, Term, Triple};
use oxiland::io::{Parser as RdfParser, Serializer, Syntax};
use oxiland::terms::{self, Literal};
use oxiland::{
    Model, OpenOptions, Query, QueryResults, ResultsFormat, StatementPattern, StorageBackend,
    serialize_query_results_to_string,
};

#[derive(Debug, Parser)]
#[command(
    name = "oxiland-cli",
    version,
    about = "Import, inspect, query, and export local RDF datasets"
)]
struct Cli {
    /// Suppress informational messages.
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Create the durable store directory if missing (required for new paths).
    #[arg(short = 'n', long)]
    new: bool,

    /// Storage type: memory, fjall, or a compiled optional backend.
    #[arg(short = 's', long, default_value = "fjall")]
    storage: String,

    /// Serialization syntax for print/serialize/find output (default nquads).
    #[arg(short = 'o', long, default_value = "nquads")]
    output: String,

    /// SPARQL results format.
    #[arg(short = 'r', long, default_value = "xml")]
    results: String,

    /// Store name: with `-s memory` use `memory`; durable backends use a path.
    store_name: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Parse the complete RDF input before inserting it into the store.
    Parse {
        path: PathBuf,
        #[arg(long)]
        syntax: Option<String>,
        #[arg(long)]
        base: Option<String>,
    },
    /// Parse RDF into the store using progressive streaming load.
    #[command(name = "parse-stream")]
    ParseStream {
        path: PathBuf,
        #[arg(long)]
        syntax: Option<String>,
        #[arg(long)]
        base: Option<String>,
    },
    /// Serialize the store.
    Serialize {
        #[arg(long)]
        syntax: Option<String>,
    },
    /// Print the store as triples/quads.
    Print,
    /// Add a statement.
    Add {
        subject: String,
        predicate: String,
        object: String,
        context: Option<String>,
    },
    /// Remove a statement.
    Remove {
        subject: String,
        predicate: String,
        object: String,
        context: Option<String>,
    },
    /// Find matching statements (`-` wildcards).
    Find {
        subject: String,
        predicate: String,
        object: String,
        context: Option<String>,
    },
    /// Run a SPARQL query (`query - - "SELECT …"` or `query sparql - "…"`).
    Query {
        /// Query language name: `-` or `sparql` only.
        name: String,
        /// Query URI or `-` (URI is ignored; SPARQL string is required).
        _uri: String,
        query: String,
    },
    /// List named graph contexts.
    Contexts,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("oxiland-cli: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let model = open_model(&cli)?;
    match cli.command {
        Commands::Parse { path, syntax, base } => {
            let n = load_path(&model, &path, syntax.as_deref(), base.as_deref(), false)?;
            if !cli.quiet {
                eprintln!("parsed {n} statement(s)");
            }
        }
        Commands::ParseStream { path, syntax, base } => {
            let n = load_path(&model, &path, syntax.as_deref(), base.as_deref(), true)?;
            if !cli.quiet {
                eprintln!("parsed {n} statement(s)");
            }
        }
        Commands::Serialize { syntax } => {
            let syntax = Syntax::from_name(syntax.as_deref().unwrap_or(&cli.output))?;
            let text = Serializer::for_syntax(syntax).serialize_model_to_string(&model)?;
            print!("{text}");
        }
        Commands::Print => {
            let syntax = Syntax::from_name(&cli.output)?;
            let text = Serializer::for_syntax(syntax).serialize_model_to_string(&model)?;
            print!("{text}");
        }
        Commands::Add {
            subject,
            predicate,
            object,
            context,
        } => {
            let triple = Triple::new(
                parse_named_or_blank(&subject)?,
                terms::named_node(&predicate)?,
                parse_term(&object)?,
            );
            let inserted = if let Some(ctx) = context {
                model.add_to_graph(triple, GraphName::NamedNode(terms::named_node(ctx)?))?
            } else {
                model.add(triple)?
            };
            if !cli.quiet {
                eprintln!("added: {inserted}");
            }
        }
        Commands::Remove {
            subject,
            predicate,
            object,
            context,
        } => {
            let triple = Triple::new(
                parse_named_or_blank(&subject)?,
                terms::named_node(&predicate)?,
                parse_term(&object)?,
            );
            let removed = if let Some(ctx) = context {
                model.remove_from_graph(triple, GraphName::NamedNode(terms::named_node(ctx)?))?
            } else {
                model.remove(triple)?
            };
            if !cli.quiet {
                eprintln!("removed: {removed}");
            }
        }
        Commands::Find {
            subject,
            predicate,
            object,
            context,
        } => {
            let subj = optional_named_or_blank(&subject)?;
            let pred = optional_named(&predicate)?;
            let obj = optional_term(&object)?;
            let graph_owned = match &context {
                Some(ctx) if ctx != "-" => Some(GraphName::NamedNode(terms::named_node(ctx)?)),
                _ => None,
            };
            let pattern = StatementPattern {
                subject: subj.as_ref().map(NamedOrBlankNode::as_ref),
                predicate: pred.as_ref().map(oxigraph::model::NamedNode::as_ref),
                object: obj.as_ref().map(Term::as_ref),
                graph_name: graph_owned.as_ref().map(GraphName::as_ref),
            };
            let quads: Result<Vec<Quad>, _> = model.find(pattern).collect();
            let quads = quads?;
            let syntax = Syntax::from_name(&cli.output)?;
            let text = Serializer::for_syntax(syntax).serialize_quads_to_string(quads)?;
            print!("{text}");
        }
        Commands::Query { name, _uri, query } => {
            validate_query_language(&name)?;
            match Query::new(&query).execute(&model)? {
                QueryResults::Boolean(value) => {
                    let format = ResultsFormat::from_name(&cli.results)?;
                    let text =
                        serialize_query_results_to_string(QueryResults::Boolean(value), format)?;
                    print!("{text}");
                }
                QueryResults::Solutions(solutions) => {
                    let format = ResultsFormat::from_name(&cli.results)?;
                    let text = serialize_query_results_to_string(
                        QueryResults::Solutions(solutions),
                        format,
                    )?;
                    print!("{text}");
                }
                QueryResults::Graph(iter) => {
                    let syntax = Syntax::from_name(&cli.output)?;
                    let tmp = Model::new()?;
                    for item in iter {
                        tmp.add(item?)?;
                    }
                    let text = Serializer::for_syntax(syntax).serialize_model_to_string(&tmp)?;
                    print!("{text}");
                }
            }
        }
        Commands::Contexts => {
            let mut names = std::collections::BTreeSet::new();
            for item in model.find(StatementPattern::default()) {
                let quad = item?;
                if let GraphName::NamedNode(node) = quad.graph_name {
                    names.insert(node.as_str().to_owned());
                }
            }
            for name in names {
                println!("{name}");
            }
        }
    }
    Ok(())
}

fn load_path(
    model: &Model,
    path: &Path,
    syntax: Option<&str>,
    base: Option<&str>,
    progressive: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let syntax = resolve_syntax(syntax, path)?;
    let mut parser = RdfParser::for_syntax(syntax);
    if let Some(base) = base {
        parser = parser.base_iri(base)?;
    }
    if progressive {
        Ok(parser.load_path_into(model, path)?)
    } else {
        Ok(parser.load_path_collecting(model, path)?)
    }
}

fn open_model(cli: &Cli) -> Result<Model, Box<dyn std::error::Error>> {
    let backend = match StorageBackend::from_name(&cli.storage) {
        Ok(backend) => backend,
        Err(err) => {
            return Err(format!("unsupported storage type: {err}").into());
        }
    };
    if backend == StorageBackend::Memory {
        if cli.store_name != "memory" && !cli.quiet {
            eprintln!(
                "oxiland-cli: ignoring store-name '{}' for memory storage",
                cli.store_name
            );
        }
        return Ok(Model::open_with(OpenOptions::new(
            StorageBackend::Memory,
            "memory",
        ))?);
    }
    if cli.store_name == "memory" {
        return Err(format!(
            "store-name 'memory' requires -s memory; use a filesystem path with -s {}",
            backend.name()
        )
        .into());
    }
    let opts = OpenOptions::new(backend, &cli.store_name).create(cli.new);
    Ok(Model::open_with(opts)?)
}

fn resolve_syntax(name: Option<&str>, path: &Path) -> Result<Syntax, Box<dyn std::error::Error>> {
    if let Some(name) = name {
        return Ok(Syntax::from_name(name)?);
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or("cannot detect syntax from path; pass --syntax")?;
    Ok(Syntax::from_extension(ext)?)
}

fn validate_query_language(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match name.trim().to_ascii_lowercase().as_str() {
        "-" | "sparql" | "sparql11" => Ok(()),
        other => Err(format!(
            "unsupported query language '{other}' (use '-' or 'sparql'; RDQL is not supported)"
        )
        .into()),
    }
}

fn parse_named_or_blank(raw: &str) -> Result<NamedOrBlankNode, Box<dyn std::error::Error>> {
    if let Some(id) = raw.strip_prefix("_:") {
        return Ok(NamedOrBlankNode::BlankNode(terms::blank_node(Some(id))?));
    }
    Ok(NamedOrBlankNode::NamedNode(terms::named_node(raw)?))
}

fn parse_term(raw: &str) -> Result<Term, Box<dyn std::error::Error>> {
    if raw.contains("^^") || raw.contains('@') && raw.starts_with('"') {
        return Err(
            "typed/language-tagged literals are not supported as CLI node arguments; use SPARQL Update or parse RDF"
                .into(),
        );
    }
    if raw.starts_with("_:") || looks_iri(raw) {
        return Ok(Term::from(parse_named_or_blank(raw)?));
    }
    let lit = raw
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw);
    Ok(Term::Literal(Literal::new_simple_literal(lit)))
}

fn optional_named_or_blank(
    raw: &str,
) -> Result<Option<NamedOrBlankNode>, Box<dyn std::error::Error>> {
    if raw == "-" {
        return Ok(None);
    }
    Ok(Some(parse_named_or_blank(raw)?))
}

fn optional_named(
    raw: &str,
) -> Result<Option<oxigraph::model::NamedNode>, Box<dyn std::error::Error>> {
    if raw == "-" {
        return Ok(None);
    }
    Ok(Some(terms::named_node(raw)?))
}

fn optional_term(raw: &str) -> Result<Option<Term>, Box<dyn std::error::Error>> {
    if raw == "-" {
        return Ok(None);
    }
    Ok(Some(parse_term(raw)?))
}

fn looks_iri(raw: &str) -> bool {
    raw.contains(':') && !raw.starts_with('"')
}
