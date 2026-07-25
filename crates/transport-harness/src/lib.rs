//! Shared startup, output, and error mechanics for workflow-tool transports.

use std::io::Write;

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// The transport being started by the harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transport {
    /// A command-line interface.
    Cli,
    /// A Model Context Protocol server over stdio.
    Mcp,
    /// An HTTP server.
    Http,
}

impl std::fmt::Display for Transport {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let name = match self {
            Self::Cli => "cli",
            Self::Mcp => "mcp",
            Self::Http => "http",
        };
        formatter.write_str(name)
    }
}

/// Shared output emitted by transport entry points.
#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    /// Human-readable output.
    Text(String),
    /// Machine-readable JSON output.
    Json(Value),
}

impl Output {
    /// Serializes a value as machine-readable JSON output.
    pub fn json(value: impl Serialize) -> Result<Self, HarnessError> {
        Ok(Self::Json(serde_json::to_value(value)?))
    }
}

/// Failures normalized at the transport boundary.
#[derive(Debug, Error)]
pub enum HarnessError {
    /// A domain command or handler failed.
    #[error("{0}")]
    Domain(String),
    /// Shared argument parsing failed.
    #[error("invalid arguments: {0}")]
    Arguments(String),
    /// Shared output could not be serialized.
    #[error("could not serialize transport output: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Shared output or server I/O failed.
    #[error("transport I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// A transport could not start or stopped unexpectedly.
    #[error("{transport} transport failed: {message}")]
    Transport {
        /// The affected transport.
        transport: Transport,
        /// The source error rendered without erasing transport context.
        message: String,
    },
}

impl HarnessError {
    /// Wraps a domain-owned error without coupling the harness to its type.
    pub fn domain(error: impl std::fmt::Display) -> Self {
        Self::Domain(error.to_string())
    }

    fn transport(
        transport: Transport,
        error: impl std::fmt::Display,
    ) -> Self {
        Self::Transport {
            transport,
            message: error.to_string(),
        }
    }
}

/// Writes shared output with exactly one trailing newline.
pub fn write_output(
    mut writer: impl Write,
    output: &Output,
) -> Result<(), HarnessError> {
    match output {
        Output::Text(text) => writeln!(writer, "{text}")?,
        Output::Json(value) => serde_json::to_writer(&mut writer, value)
            .and_then(|()| serde_json::to_writer(&mut writer, "\n"))?,
    }
    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .try_init();
}

/// CLI parsing, dispatch, and output mechanics.
#[cfg(feature = "cli")]
pub mod cli {
    pub use clap;
    use clap::Parser;

    use super::{HarnessError, Output, Transport, init_tracing, write_output};

    /// Parses process arguments, invokes domain dispatch, and writes its output.
    pub fn run<Command, Dispatch>(
        dispatch: Dispatch
    ) -> Result<(), HarnessError>
    where
        Command: Parser,
        Dispatch: FnOnce(Command) -> Result<Output, HarnessError>,
    {
        run_from(std::env::args_os(), std::io::stdout().lock(), dispatch)
    }

    /// Parses supplied arguments and writes output to an injected writer.
    pub fn run_from<Command, Arguments, Argument, Writer, Dispatch>(
        arguments: Arguments,
        writer: Writer,
        dispatch: Dispatch,
    ) -> Result<(), HarnessError>
    where
        Command: Parser,
        Arguments: IntoIterator<Item = Argument>,
        Argument: Into<std::ffi::OsString> + Clone,
        Writer: std::io::Write,
        Dispatch: FnOnce(Command) -> Result<Output, HarnessError>,
    {
        init_tracing();
        let command = Command::try_parse_from(arguments)
            .map_err(|error| HarnessError::Arguments(error.to_string()))?;
        let output = dispatch(command)?;
        write_output(writer, &output)
    }

    /// Creates an error carrying CLI transport context.
    pub fn startup_error(error: impl std::fmt::Display) -> HarnessError {
        HarnessError::transport(Transport::Cli, error)
    }
}

/// MCP stdio server startup mechanics.
#[cfg(feature = "mcp")]
pub mod mcp {
    pub use rmcp;
    use rmcp::{ServerHandler, ServiceExt, transport::stdio};

    use super::{HarnessError, Transport, init_tracing};

    /// Serves a domain-owned MCP handler over stdio until the peer disconnects.
    pub fn run<Server>(server: Server) -> Result<(), HarnessError>
    where
        Server: ServerHandler + Send + 'static,
    {
        init_tracing();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        runtime.block_on(async move {
            let service = server.serve(stdio()).await.map_err(|error| {
                HarnessError::transport(Transport::Mcp, error)
            })?;
            service.waiting().await.map_err(|error| {
                HarnessError::transport(Transport::Mcp, error)
            })?;
            Ok(())
        })
    }
}

/// HTTP listener startup and error response mechanics.
#[cfg(feature = "http")]
pub mod http {
    use std::net::SocketAddr;

    pub use axum;
    use axum::{
        Json,
        response::{IntoResponse, Response},
    };
    pub use axum::{Router, http::StatusCode};
    use serde::Serialize;

    use super::{HarnessError, Transport, init_tracing};

    /// A stable JSON error envelope for domain HTTP handlers.
    #[derive(Clone, Debug, Serialize)]
    pub struct HttpError {
        #[serde(skip)]
        status: StatusCode,
        /// Stable machine-readable error code.
        pub code: String,
        /// Human-readable error detail.
        pub message: String,
    }

    impl HttpError {
        /// Creates a structured HTTP error response.
        pub fn new(
            status: StatusCode,
            code: impl Into<String>,
            message: impl Into<String>,
        ) -> Self {
            Self {
                status,
                code: code.into(),
                message: message.into(),
            }
        }

        /// Returns the HTTP status associated with this error.
        pub fn status(&self) -> StatusCode {
            self.status
        }
    }

    impl IntoResponse for HttpError {
        fn into_response(self) -> Response {
            (self.status, Json(self)).into_response()
        }
    }

    /// Binds an address and serves a domain-owned router until shutdown.
    pub fn run(
        address: SocketAddr,
        router: Router,
    ) -> Result<(), HarnessError> {
        init_tracing();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        runtime.block_on(async move {
            let listener =
                tokio::net::TcpListener::bind(address).await.map_err(
                    |error| HarnessError::transport(Transport::Http, error),
                )?;
            tracing::info!(%address, "HTTP transport listening");
            axum::serve(listener, router).await.map_err(|error| {
                HarnessError::transport(Transport::Http, error)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{HarnessError, Output, Transport, write_output};

    #[derive(Serialize)]
    struct Payload {
        status: &'static str,
    }

    #[test]
    fn write_output_appends_one_newline_to_text() {
        let mut bytes = Vec::new();
        write_output(&mut bytes, &Output::Text("ready".into()))
            .expect("text output should write");
        assert_eq!(bytes, b"ready\n");
    }

    #[test]
    fn output_json_serializes_domain_values() {
        let output = Output::json(Payload { status: "ready" })
            .expect("payload should serialize");
        assert_eq!(
            output,
            Output::Json(serde_json::json!({"status": "ready"}))
        );
    }

    #[test]
    fn transport_error_retains_transport_context() {
        let error =
            HarnessError::transport(Transport::Mcp, "connection closed");
        assert_eq!(
            error.to_string(),
            "mcp transport failed: connection closed"
        );
    }

    #[cfg(feature = "cli")]
    #[test]
    fn cli_run_from_dispatches_parsed_domain_command() {
        use crate::cli::clap::{self, Parser};

        #[derive(Parser)]
        struct Command {
            #[arg(long)]
            name: String,
        }

        let mut bytes = Vec::new();
        crate::cli::run_from(
            ["example", "--name", "Ada"],
            &mut bytes,
            |command: Command| {
                Ok(Output::Text(format!("hello {}", command.name)))
            },
        )
        .expect("CLI dispatch should succeed");

        assert_eq!(bytes, b"hello Ada\n");
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_error_retains_the_mapped_status() {
        let error = crate::http::HttpError::new(
            crate::http::StatusCode::NOT_FOUND,
            "not_found",
            "missing",
        );
        assert_eq!(error.status(), crate::http::StatusCode::NOT_FOUND);
    }
}
