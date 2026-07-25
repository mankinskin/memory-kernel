//! Reference-proof integration tests for the shared transport harness.
//!
//! Design: memory-kernel spec `transport-harness`, section
//! "Reference-proof integration tests (design)" (ticket 60114a17); implements
//! ticket 2cc7680c.
//!
//! One realistic domain operation (`describe`) is exposed through CLI, MCP, and
//! HTTP. Each transport proves BOTH the success output shape AND the domain
//! error path — the harness error envelope and, for HTTP, the status mapping.
//! The fixture domain lives inline here (dev-only) so the library surface and
//! its `default = []` slimness are untouched.

/// Smallest realistic domain operation shared by every transport.
///
/// Only compiled when a transport feature is active; under `default = []` this
/// module — and every transport proof below — is absent, which is itself the
/// slimness proof.
#[cfg(any(feature = "cli", feature = "mcp", feature = "http"))]
mod fixture {
    use serde::Serialize;

    /// A described item returned on the success path.
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct Item {
        pub id: String,
        pub summary: String,
    }

    /// The single domain error, rendered as `unknown item: <id>`.
    #[derive(Debug)]
    pub struct NotFound(pub String);

    impl std::fmt::Display for NotFound {
        fn fmt(
            &self,
            formatter: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            write!(formatter, "unknown item: {}", self.0)
        }
    }

    /// Resolves an id against a tiny fixed registry.
    pub fn describe(id: &str) -> Result<Item, NotFound> {
        match id {
            "harness" => Ok(Item {
                id: "harness".to_string(),
                summary: "Shared transport harness".to_string(),
            }),
            other => Err(NotFound(other.to_string())),
        }
    }
}

/// Core harness behavior usable without any transport feature.
///
/// This runs even under `default = []`, proving the shared output and error
/// mechanics do not depend on CLI, MCP, or HTTP.
mod core_proof {
    use transport_harness::{
        HarnessError,
        Output,
        Transport,
        write_output,
    };

    #[test]
    fn core_output_and_error_work_without_transport_features() {
        let mut text = Vec::new();
        write_output(&mut text, &Output::Text("ready".to_string()))
            .expect("text output should write");
        assert_eq!(text, b"ready\n");

        let mut json = Vec::new();
        write_output(
            &mut json,
            &Output::json(serde_json::json!({"status": "ready"}))
                .expect("json should serialize"),
        )
        .expect("json output should write");
        assert_eq!(json, b"{\"status\":\"ready\"}\n");

        assert_eq!(HarnessError::domain("boom").to_string(), "boom");
        assert_eq!(Transport::Http.to_string(), "http");
    }
}

/// CLI transport: parses a domain subcommand and dispatches through shared
/// output and error handling.
#[cfg(feature = "cli")]
mod cli_proof {
    use transport_harness::{
        HarnessError,
        Output,
        cli::{
            self,
            clap::{
                self,
                Parser,
            },
        },
    };

    use super::fixture;

    #[derive(Parser)]
    #[command(name = "example-cli")]
    struct Command {
        #[command(subcommand)]
        op: Op,
    }

    #[derive(clap::Subcommand)]
    enum Op {
        /// Describe an item by id.
        Describe {
            #[arg(long)]
            id: String,
        },
    }

    fn dispatch(command: Command) -> Result<Output, HarnessError> {
        match command.op {
            Op::Describe { id } => {
                let item =
                    fixture::describe(&id).map_err(HarnessError::domain)?;
                Output::json(item)
            }
        }
    }

    #[test]
    fn cli_describe_success_emits_one_json_line() {
        let mut buffer = Vec::new();
        cli::run_from(
            ["example-cli", "describe", "--id", "harness"],
            &mut buffer,
            dispatch,
        )
        .expect("cli dispatch should succeed");
        assert_eq!(
            buffer,
            b"{\"id\":\"harness\",\"summary\":\"Shared transport harness\"}\n"
        );
    }

    #[test]
    fn cli_describe_unknown_id_returns_domain_error() {
        let mut buffer = Vec::new();
        let error = cli::run_from(
            ["example-cli", "describe", "--id", "missing"],
            &mut buffer,
            dispatch,
        )
        .expect_err("unknown id should fail");
        assert!(matches!(error, HarnessError::Domain(_)));
        assert_eq!(error.to_string(), "unknown item: missing");
        assert!(buffer.is_empty());
    }

    #[test]
    fn cli_invalid_arguments_map_to_arguments_error() {
        let mut buffer = Vec::new();
        let error =
            cli::run_from(["example-cli", "bogus"], &mut buffer, dispatch)
                .expect_err("invalid subcommand should fail");
        assert!(matches!(error, HarnessError::Arguments(_)));
    }
}

/// MCP transport: registers a domain tool and invokes it in-process for both
/// the success and the domain-error paths.
#[cfg(feature = "mcp")]
mod mcp_proof {
    use serde::Deserialize;
    use transport_harness::mcp::rmcp::{
        ErrorData as McpError,
        ServerHandler,
        handler::server::{
            tool::ToolRouter,
            wrapper::Parameters,
        },
        model::{
            CallToolResult,
            Content,
            RawContent,
        },
        schemars::{
            self,
            JsonSchema,
        },
        tool,
        tool_handler,
        tool_router,
    };

    use super::fixture;

    #[derive(Clone)]
    struct DescribeServer {
        tool_router: ToolRouter<Self>,
    }

    impl DescribeServer {
        fn new() -> Self {
            Self {
                tool_router: Self::tool_router(),
            }
        }
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    struct DescribeArgs {
        /// Item id to describe.
        id: String,
    }

    #[tool_router]
    impl DescribeServer {
        /// Describe an item by id.
        #[tool(description = "Describe an item by id")]
        async fn describe(
            &self,
            Parameters(args): Parameters<DescribeArgs>,
        ) -> Result<CallToolResult, McpError> {
            let item = fixture::describe(&args.id)
                .map_err(|error| McpError::invalid_params(error.to_string(), None))?;
            let json = serde_json::to_string(&serde_json::json!({
                "id": item.id,
                "summary": item.summary,
            }))
            .expect("item should serialize");
            Ok(CallToolResult::success(vec![Content::text(json)]))
        }
    }

    #[tool_handler]
    impl ServerHandler for DescribeServer {}

    fn text_of(result: &CallToolResult) -> String {
        let content =
            result.content.first().expect("result should carry content");
        match &content.raw {
            RawContent::Text(text) => text.text.clone(),
            other => panic!("expected text content, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn mcp_describe_tool_success() {
        let server = DescribeServer::new();
        let result = server
            .describe(Parameters(DescribeArgs {
                id: "harness".to_string(),
            }))
            .await
            .expect("tool call should succeed");
        assert_eq!(
            text_of(&result),
            "{\"id\":\"harness\",\"summary\":\"Shared transport harness\"}"
        );
    }

    #[tokio::test]
    async fn mcp_describe_tool_unknown_id_errors() {
        let server = DescribeServer::new();
        let error = server
            .describe(Parameters(DescribeArgs {
                id: "missing".to_string(),
            }))
            .await
            .expect_err("unknown id should error");
        assert!(format!("{error:?}").contains("unknown item: missing"));
    }
}

/// HTTP transport: registers a domain success route and a domain error route
/// that maps through the shared `HttpError` envelope and status code.
#[cfg(feature = "http")]
mod http_proof {
    use tower::ServiceExt;
    use transport_harness::http::{
        HttpError,
        Router,
        StatusCode,
        axum::{
            Json,
            body::{
                Body,
                to_bytes,
            },
            extract::Path,
            http::Request,
            response::{
                IntoResponse,
                Response,
            },
            routing::get,
        },
    };

    use super::fixture;

    async fn describe(Path(id): Path<String>) -> Response {
        match fixture::describe(&id) {
            Ok(item) => Json(serde_json::json!({
                "id": item.id,
                "summary": item.summary,
            }))
            .into_response(),
            Err(error) => HttpError::new(
                StatusCode::NOT_FOUND,
                "not_found",
                error.to_string(),
            )
            .into_response(),
        }
    }

    fn router() -> Router {
        Router::new().route("/describe/{id}", get(describe))
    }

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        serde_json::from_slice(&bytes).expect("body should be json")
    }

    #[tokio::test]
    async fn http_describe_success_returns_item() {
        let response = router()
            .oneshot(
                Request::builder()
                    .uri("/describe/harness")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_json(response).await,
            serde_json::json!({
                "id": "harness",
                "summary": "Shared transport harness"
            })
        );
    }

    #[tokio::test]
    async fn http_describe_unknown_id_maps_to_not_found_envelope() {
        let response = router()
            .oneshot(
                Request::builder()
                    .uri("/describe/missing")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_json(response).await,
            serde_json::json!({
                "code": "not_found",
                "message": "unknown item: missing"
            })
        );
    }
}
