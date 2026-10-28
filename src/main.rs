//! `a-graph`: reads the yml of a graph, checks it and runs it.
//!
//! ```text
//! a-graph graph.yml "the message"       # checks the file and runs it
//! cat question.txt | a-graph graph.yml  # without a message, it is read from the standard input
//! a-graph --check graph.yml             # only checks the file
//! a-graph --print-schema                # the JSON Schema of the graph file
//! a-graph --print-docs                  # the reference of the graph file, in Markdown
//! a-graph --print-service-schema        # the JSON Schema of the file of the service
//! a-graph --print-service-docs          # the reference of the file of the service
//! a-graph --print-service-api           # the OpenAPI description of the service
//! ```
//!
//! The result goes to the standard output (a JSON document, or `node: text` lines with
//! `--chat_mode`); errors, logs and the display of `verbose` go to the standard error.
//! Exit codes: 0 well; 1 the yml is not valid; 2 wrong use; 3 the run failed.

use std::io::{IsTerminal, Read};
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;

const INVALID: u8 = 1;
const USAGE: u8 = 2;
const FAILED: u8 = 3;

#[derive(Parser)]
#[command(name = "a-graph", version, about = "Checks and runs a graph of agents written in a yml file")]
struct Cli {
    /// The yml file of the graph (with `--serve`: the configuration of the service).
    #[arg(required_unless_present_any = ["print_schema", "print_docs", "print_service_schema", "print_service_docs", "print_service_api"])]
    file: Option<std::path::PathBuf>,
    /// The message of the user. Without it, the standard input is read.
    message: Option<String>,
    /// Only check the file: nothing is run.
    #[arg(long)]
    check: bool,
    /// Start the service. The file is then the configuration of the service, not a graph.
    #[arg(long, conflicts_with_all = ["check", "chat_mode", "message"])]
    serve: bool,
    /// Write only `node: text` for each result, instead of the JSON document.
    #[arg(long = "chat_mode")]
    chat_mode: bool,
    /// Print the JSON Schema of the graph file (`spec/graph.schema.json`).
    #[arg(long, conflicts_with_all = ["file", "message", "check", "serve", "chat_mode", "print_docs"])]
    print_schema: bool,
    /// Print the reference of the graph file, in Markdown (`docs/graph.md`).
    #[arg(long, conflicts_with_all = ["file", "message", "check", "serve", "chat_mode", "print_service_schema", "print_service_docs"])]
    print_docs: bool,
    /// Print the JSON Schema of the file of the service (`spec/service.schema.json`).
    #[arg(long, conflicts_with_all = ["file", "message", "check", "serve", "chat_mode", "print_schema", "print_service_docs"])]
    print_service_schema: bool,
    /// Print the reference of the file of the service, in Markdown (`docs/service.md`).
    #[arg(long, conflicts_with_all = ["file", "message", "check", "serve", "chat_mode", "print_schema", "print_service_api"])]
    print_service_docs: bool,
    /// Print the OpenAPI description of the HTTP service (`spec/service.openapi.json`).
    #[arg(long, conflicts_with_all = ["file", "message", "check", "serve", "chat_mode", "print_schema", "print_docs", "print_service_schema"])]
    print_service_api: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    if cli.print_schema {
        print!("{}", a_graph::schema::json_schema());
        return ExitCode::SUCCESS;
    }
    if cli.print_docs {
        print!("{}", a_graph::schema::reference());
        return ExitCode::SUCCESS;
    }
    if cli.print_service_schema {
        print!("{}", a_graph::schema::service_json_schema());
        return ExitCode::SUCCESS;
    }
    if cli.print_service_docs {
        print!("{}", a_graph::schema::service_reference());
        return ExitCode::SUCCESS;
    }
    if cli.print_service_api {
        print!("{}", a_graph::service::openapi());
        return ExitCode::SUCCESS;
    }
    // clap requires the file without the two options above
    let Some(file) = cli.file else { return ExitCode::from(USAGE) };

    let source = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("a-graph: cannot read `{}`: {e}", file.display());
            return ExitCode::from(USAGE);
        }
    };
    if cli.serve {
        return serve(&source).await;
    }
    let program = match a_graph::run::build(&source) {
        Ok(p) => p,
        Err(errors) => {
            eprintln!("{errors}");
            return ExitCode::from(INVALID);
        }
    };
    if cli.check {
        return ExitCode::SUCCESS;
    }

    let message = match cli.message {
        Some(m) => m,
        None if std::io::stdin().is_terminal() => {
            eprintln!("a-graph: no message: give it as the second argument or on the standard input");
            return ExitCode::from(USAGE);
        }
        None => {
            let mut text = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut text) {
                eprintln!("a-graph: cannot read the standard input: {e}");
                return ExitCode::from(USAGE);
            }
            text.trim_end_matches(['\n', '\r']).to_string()
        }
    };

    let program = Arc::new(program);
    let outcome = program.run(&message).await;
    // the log of a network output delivers what is still queued before the program ends
    if let Some(p) = Arc::into_inner(program) {
        p.logger.close().await;
    }
    match outcome {
        Ok(output) => {
            if cli.chat_mode {
                print!("{}", output.to_chat_text());
            } else {
                println!("{}", output.to_json());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(FAILED)
        }
    }
}

/// Starts the service described by the text of its configuration file.
async fn serve(source: &str) -> ExitCode {
    let config = match a_graph::service::ServiceConfig::read(source) {
        Ok(c) => c,
        Err(errors) => {
            eprintln!("{errors}");
            return ExitCode::from(INVALID);
        }
    };
    eprintln!("a-graph: listening on port {}", config.port);
    match a_graph::service::serve(config, stop()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("a-graph: {e}");
            ExitCode::from(FAILED)
        }
    }
}

/// Ready when the service is told to stop: `SIGINT` (Ctrl-C) or, on unix, `SIGTERM` (what `docker stop` sends).
/// The handler of `SIGTERM` is set at once, before the service listens, so no signal is missed. A signal that
/// cannot be listened for is written and never comes.
fn stop() -> impl Future<Output = ()> + Send + 'static {
    #[cfg(unix)]
    let terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate());
    async move {
        let interrupt = async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                eprintln!("a-graph: cannot listen for SIGINT: {e}");
                std::future::pending::<()>().await;
            }
        };
        #[cfg(unix)]
        let terminate = async {
            match terminate {
                Ok(mut s) => {
                    s.recv().await;
                }
                Err(e) => {
                    eprintln!("a-graph: cannot listen for SIGTERM: {e}");
                    std::future::pending::<()>().await;
                }
            }
        };
        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();
        tokio::select! {
            _ = interrupt => {}
            _ = terminate => {}
        }
    }
}
