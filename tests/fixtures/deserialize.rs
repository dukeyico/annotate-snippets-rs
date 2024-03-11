use serde::{Deserialize, Deserializer, Serialize};
use std::ops::Range;

use annotate_snippets::{renderer::Margin, Annotation, Label, Level, Message, Renderer, Snippet};

#[derive(Deserialize)]
pub struct Fixture<'a> {
    #[serde(default)]
    pub renderer: RendererDef,
    #[serde(borrow)]
    pub message: MessageDef<'a>,
}

#[derive(Deserialize)]
pub struct MessageDef<'a> {
    #[serde(deserialize_with = "deserialize_label")]
    #[serde(borrow)]
    pub title: Label<'a>,
    #[serde(default)]
    #[serde(borrow)]
    pub id: Option<&'a str>,
    #[serde(deserialize_with = "deserialize_labels")]
    #[serde(default)]
    #[serde(borrow)]
    pub footer: Vec<Label<'a>>,
    #[serde(deserialize_with = "deserialize_snippets")]
    #[serde(borrow)]
    pub snippets: Vec<Snippet<'a>>,
}

impl<'a> From<MessageDef<'a>> for Message<'a> {
    fn from(val: MessageDef<'a>) -> Self {
        let MessageDef {
            title,
            id,
            footer,
            snippets,
        } = val;
        let mut message = Message::title(title);
        if let Some(id) = id {
            message = message.id(id);
        }
        message = snippets
            .into_iter()
            .fold(message, |message, snippet| message.snippet(snippet));
        message = footer
            .into_iter()
            .fold(message, |message, label| message.footer(label));
        message
    }
}

fn deserialize_label<'de, D>(deserializer: D) -> Result<Label<'de>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper<'a>(
        #[serde(with = "LabelDef")]
        #[serde(borrow)]
        LabelDef<'a>,
    );

    Wrapper::deserialize(deserializer).map(|Wrapper(label)| Label::new(label.level, label.label))
}

fn deserialize_labels<'de, D>(deserializer: D) -> Result<Vec<Label<'de>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper<'a>(
        #[serde(with = "LabelDef")]
        #[serde(borrow)]
        LabelDef<'a>,
    );

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter()
        .map(|Wrapper(a)| Label::new(a.level, a.label))
        .collect())
}

fn deserialize_snippets<'de, D>(deserializer: D) -> Result<Vec<Snippet<'de>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper<'a>(
        #[serde(with = "SnippetDef")]
        #[serde(borrow)]
        SnippetDef<'a>,
    );

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| a.into()).collect())
}

#[derive(Deserialize)]
pub struct SnippetDef<'a> {
    #[serde(borrow)]
    pub source: &'a str,
    pub line_start: usize,
    #[serde(borrow)]
    pub origin: Option<&'a str>,
    #[serde(deserialize_with = "deserialize_annotations")]
    #[serde(borrow)]
    pub annotations: Vec<Annotation<'a>>,
    #[serde(default)]
    pub fold: bool,
}

impl<'a> From<SnippetDef<'a>> for Snippet<'a> {
    fn from(val: SnippetDef<'a>) -> Self {
        let SnippetDef {
            source,
            line_start,
            origin,
            annotations,
            fold,
        } = val;
        let mut snippet = Snippet::new(source).line_start(line_start).fold(fold);
        if let Some(origin) = origin {
            snippet = snippet.origin(origin)
        }
        snippet = annotations
            .into_iter()
            .fold(snippet, |snippet, annotation| {
                snippet.annotation(annotation)
            });
        snippet
    }
}

fn deserialize_annotations<'de, D>(deserializer: D) -> Result<Vec<Annotation<'de>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper<'a>(#[serde(borrow)] AnnotationDef<'a>);

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| a.into()).collect())
}

#[derive(Serialize, Deserialize)]
pub struct AnnotationDef<'a> {
    pub range: Range<usize>,
    #[serde(borrow)]
    pub label: &'a str,
    #[serde(with = "LevelDef")]
    pub level: Level,
}

impl<'a> From<AnnotationDef<'a>> for Annotation<'a> {
    fn from(val: AnnotationDef<'a>) -> Self {
        let AnnotationDef {
            range,
            label,
            level,
        } = val;
        Label::new(level, label).span(range)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LabelDef<'a> {
    #[serde(with = "LevelDef")]
    pub level: Level,
    #[serde(borrow)]
    pub label: &'a str,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(remote = "Level")]
enum LevelDef {
    Error,
    Warning,
    Info,
    Note,
    Help,
}

#[derive(Default, Deserialize)]
pub struct RendererDef {
    #[serde(default)]
    anonymized_line_numbers: bool,
    #[serde(deserialize_with = "deserialize_margin")]
    #[serde(default)]
    margin: Option<Margin>,
}

impl From<RendererDef> for Renderer {
    fn from(val: RendererDef) -> Self {
        let RendererDef {
            anonymized_line_numbers,
            margin,
        } = val;
        Renderer::plain()
            .anonymized_line_numbers(anonymized_line_numbers)
            .margin(margin)
    }
}

fn deserialize_margin<'de, D>(deserializer: D) -> Result<Option<Margin>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper {
        whitespace_left: usize,
        span_left: usize,
        span_right: usize,
        label_right: usize,
        column_width: usize,
        max_line_len: usize,
    }

    Option::<Wrapper>::deserialize(deserializer).map(|opt_wrapped: Option<Wrapper>| {
        opt_wrapped.map(|wrapped: Wrapper| {
            let Wrapper {
                whitespace_left,
                span_left,
                span_right,
                label_right,
                column_width,
                max_line_len,
            } = wrapped;
            Margin::new(
                whitespace_left,
                span_left,
                span_right,
                label_right,
                column_width,
                max_line_len,
            )
        })
    })
}
