use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn render_markdown(content: &str) -> Vec<Line<'static>> {
    let parser = Parser::new(content);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_line_spans: Vec<Span<'static>> = Vec::new();
    let mut current_style = Style::default();
    let mut in_code_block = false;
    let mut code_block_lines: Vec<String> = Vec::new();
    let mut list_depth: usize = 0;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    current_line_spans.clear();
                    current_style = Style::default();
                }
                Tag::Heading { .. } => {
                    current_line_spans.clear();
                    current_style = Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD);
                }
                Tag::BlockQuote(_) => {
                    current_style = Style::default().fg(Color::DarkGray);
                    current_line_spans
                        .push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
                }
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                    code_block_lines.clear();
                }
                Tag::List(_) => {
                    list_depth += 1;
                }
                Tag::Item => {
                    let indent = "  ".repeat(list_depth.saturating_sub(1));
                    current_line_spans.push(Span::raw(format!("{}• ", indent)));
                }
                Tag::Emphasis => {
                    current_style = current_style.add_modifier(Modifier::ITALIC);
                }
                Tag::Strong => {
                    current_style = current_style.add_modifier(Modifier::BOLD);
                }
                Tag::Strikethrough => {
                    current_style = current_style.add_modifier(Modifier::CROSSED_OUT);
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                    lines.push(Line::from(""));
                }
                TagEnd::Heading(_) => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                    lines.push(Line::from(""));
                    current_style = Style::default();
                }
                TagEnd::BlockQuote(_) => {
                    current_style = Style::default();
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    // Render code block with background
                    for code_line in &code_block_lines {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", code_line),
                            Style::default().fg(Color::White).bg(Color::DarkGray),
                        )));
                    }
                    lines.push(Line::from(""));
                    code_block_lines.clear();
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    if list_depth == 0 {
                        lines.push(Line::from(""));
                    }
                }
                TagEnd::Item => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                    current_style = Style::default();
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_block_lines.push(text.to_string());
                } else {
                    current_line_spans.push(Span::styled(text.to_string(), current_style));
                }
            }
            Event::Code(code) => {
                current_line_spans.push(Span::styled(
                    code.to_string(),
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray),
                ));
            }
            Event::SoftBreak | Event::HardBreak => {
                if !current_line_spans.is_empty() {
                    lines.push(Line::from(current_line_spans.clone()));
                    current_line_spans.clear();
                }
            }
            Event::Rule => {
                lines.push(Line::from(Span::styled(
                    "─".repeat(40),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }
            _ => {}
        }
    }

    // Remove trailing empty lines
    while lines
        .last()
        .is_some_and(|l| l.spans.is_empty() || l.spans.iter().all(|s| s.content.is_empty()))
    {
        lines.pop();
    }

    if lines.is_empty() {
        lines.push(Line::from(""));
    }

    lines
}
