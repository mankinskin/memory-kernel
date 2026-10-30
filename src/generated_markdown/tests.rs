use pretty_assertions::assert_eq;

use super::{
    GeneratedMarkdownConfig, GeneratedMarkdownSnippet, ParseGeneratedMarkdownError,
    parse_generated_artifact, prepare_generated_output, render_markdown_file,
};

fn rule_like_config() -> GeneratedMarkdownConfig<'static> {
    GeneratedMarkdownConfig::new("<!-- generated:file true -->", "generated:entry")
}

#[test]
fn render_markdown_file_emits_provenance_comments_and_trimmed_blocks() {
    let rendered = render_markdown_file(
        &[
            GeneratedMarkdownSnippet::new(
                "one",
                Some("shared/agents/opening"),
                "Start with the concrete anchor.\n",
            ),
            GeneratedMarkdownSnippet::new(
                "two",
                Some("shared/agents/validation"),
                "Run the focused check next.",
            ),
        ],
        &rule_like_config(),
    );

    assert_eq!(
        rendered,
        "<!-- generated:file true -->\n\n<!-- generated:entry id=one slug=shared/agents/opening -->\nStart with the concrete anchor.\n\n<!-- generated:entry id=two slug=shared/agents/validation -->\nRun the focused check next.\n"
    );
}

#[test]
fn render_markdown_file_keeps_frontmatter_first_and_emits_provenance() {
    let rendered = render_markdown_file(
        &[GeneratedMarkdownSnippet::new(
            "prompt",
            Some("context-engine/prompts/spec"),
            "---\nname: spec\n---\nCreate a new spec entry.\n",
        )],
        &rule_like_config(),
    );

    assert_eq!(
        rendered,
        "---\nname: spec\n---\n\n<!-- generated:file true -->\n\n<!-- generated:entry id=prompt slug=context-engine/prompts/spec -->\nCreate a new spec entry.\n"
    );
}

#[test]
fn prepare_generated_output_preserves_existing_crlf_style() {
    let prepared = prepare_generated_output(
        "first\nsecond\nthird\n",
        Some("old\r\ncontent\r\nblock\r\n"),
    );

    assert_eq!(prepared, "first\r\nsecond\r\nthird\r\n");
}

#[test]
fn prepare_generated_output_reuses_existing_mixed_newline_sequence() {
    let prepared =
        prepare_generated_output("first\nsecond\nthird\n", Some("old\r\ncontent\nblock\r\n"));

    assert_eq!(prepared, "first\r\nsecond\nthird\r\n");
}

#[test]
fn prepare_generated_output_normalizes_new_files_to_lf() {
    let prepared = prepare_generated_output("first\r\nsecond\r\nthird\n", None);

    assert_eq!(prepared, "first\nsecond\nthird\n");
}

#[test]
fn parse_generated_artifact_restores_frontmatter_to_first_entry() {
    let rendered = render_markdown_file(
        &[GeneratedMarkdownSnippet::new(
            "prompt",
            Some("context-engine/prompts/spec"),
            "---\nname: spec\n---\nCreate a new spec entry.\n",
        )],
        &rule_like_config(),
    );

    let parsed =
        parse_generated_artifact(&rendered, &rule_like_config()).expect("parse generated artifact");
    assert_eq!(parsed.entries.len(), 1);
    assert_eq!(parsed.entries[0].id, "prompt");
    assert_eq!(
        parsed.entries[0].body,
        "---\nname: spec\n---\nCreate a new spec entry."
    );
}

#[test]
fn parse_generated_artifact_rejects_non_generated_file() {
    let error =
        parse_generated_artifact("# not generated", &rule_like_config()).expect_err("must fail");
    assert_eq!(
        error,
        ParseGeneratedMarkdownError::MissingGeneratedFileComment
    );
}

#[test]
fn parse_generated_artifact_handles_crlf_input_and_trimmed_bodies() {
    let input = concat!(
        "<!-- generated:file true -->\r\n\r\n",
        "<!-- generated:entry id=one slug=shared/a -->\r\n",
        "alpha\r\n\r\n",
        "<!-- generated:entry id=two slug=shared/b -->\r\n",
        "beta\r\n",
    );

    let parsed =
        parse_generated_artifact(input, &rule_like_config()).expect("parse generated artifact");
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(parsed.entries[0].body, "alpha");
    assert_eq!(parsed.entries[1].body, "beta");
}
