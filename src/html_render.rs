use ego_tree::{NodeRef, iter::Children};
use ratatui::text::Line;
use scraper::{Html, Node};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackableModifier {
    KeepPrefixSpace = 1 << 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ExclusiveModifier {
    #[default]
    Inline,
    RequiresSpace,
    NewParagraph,
    UnorderedList,
    OrderedList(u16),
}

#[derive(Debug, Clone, Copy, Default)]
struct Context {
    exclusive_modifier: ExclusiveModifier,
    stackable_modifiers: u8,

    indent: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderStatus {
    NotRendered,
    Rendered,
    RenderedRequiresSpace,
}

impl ExclusiveModifier {
    fn precedence(&self) -> u8 {
        match self {
            ExclusiveModifier::Inline => 0,
            ExclusiveModifier::RequiresSpace => 1,
            ExclusiveModifier::NewParagraph => 2,
            ExclusiveModifier::UnorderedList => 3,
            ExclusiveModifier::OrderedList(_) => 3,
        }
    }
}

impl Context {
    fn merge_exclusive_modifier(&self, modifier: ExclusiveModifier) -> Self {
        if self.exclusive_modifier.precedence() > modifier.precedence() {
            Context {
                exclusive_modifier: self.exclusive_modifier,
                stackable_modifiers: self.stackable_modifiers,
                indent: self.indent,
            }
        } else {
            let indent = match modifier {
                ExclusiveModifier::UnorderedList | ExclusiveModifier::OrderedList(_) => {
                    self.indent + 1
                }
                _ => self.indent,
            };

            Context {
                exclusive_modifier: modifier,
                stackable_modifiers: self.stackable_modifiers,
                indent,
            }
        }
    }

    fn set_exclusive_modifier(&self, modifier: ExclusiveModifier) -> Self {
        Context {
            exclusive_modifier: modifier,
            stackable_modifiers: self.stackable_modifiers,
            indent: self.indent,
        }
    }

    fn has_stackable_modifier(&self, modifier: StackableModifier) -> bool {
        self.stackable_modifiers & modifier as u8 > 0
    }

    fn add_stackable_modifier(&self, modifier: StackableModifier) -> Self {
        Context {
            exclusive_modifier: self.exclusive_modifier,
            stackable_modifiers: self.stackable_modifiers | modifier as u8,
            indent: self.indent,
        }
    }

    fn remove_stackable_modifier(&self, modifier: StackableModifier) -> Self {
        Context {
            exclusive_modifier: self.exclusive_modifier,
            stackable_modifiers: self.stackable_modifiers & !(modifier as u8),
            indent: self.indent,
        }
    }
}

impl RenderStatus {
    fn is_rendered(&self) -> bool {
        *self != RenderStatus::NotRendered
    }
}

#[derive(Debug)]
struct Renderer {
    lines: Vec<Line<'static>>,
    last_line_width: usize,

    max_width: usize,
}

pub fn render(html: &str, max_width: usize) -> Vec<Line<'static>> {
    let tree = Html::parse_document(html);
    let renderer = Renderer::new(max_width);
    renderer.render(tree)
}

impl Renderer {
    fn new(max_width: usize) -> Self {
        Self {
            lines: vec![Line::default()],
            last_line_width: 0,
            max_width,
        }
    }

    fn render(mut self, tree: Html) -> Vec<Line<'static>> {
        let root = tree.tree.root();
        self.render_node(Context::default(), root);
        self.lines
    }

    fn render_node(&mut self, ctx: Context, node: NodeRef<'_, Node>) -> RenderStatus {
        match node.value() {
            Node::Document => self.render_children(ctx, node.children()),
            Node::Fragment => self.render_children(ctx, node.children()),
            Node::Text(text) => self.render_text(ctx, &text.text),
            Node::Element(element) => match element.name() {
                "script" | "head" | "noscript" | "img" | "picutre" | "audio" | "video"
                | "source" => RenderStatus::NotRendered, // ignore
                "span" => {
                    self.render_context(ctx, first_char(node));
                    self.render_children(
                        ctx.set_exclusive_modifier(ExclusiveModifier::Inline),
                        node.children(),
                    );

                    RenderStatus::RenderedRequiresSpace
                }
                "a" => {
                    self.render_text(
                        ctx.merge_exclusive_modifier(ExclusiveModifier::RequiresSpace),
                        "[",
                    );

                    let ctx = ctx.set_exclusive_modifier(ExclusiveModifier::Inline);
                    self.render_children(ctx, node.children());
                    self.render_text(ctx, "]");
                    self.render_text(ctx, "(");
                    self.render_text(ctx, element.attr("href").unwrap_or(""));
                    self.render_text(ctx, ")");

                    RenderStatus::RenderedRequiresSpace
                }
                "strong" => {
                    self.render_text(
                        ctx.merge_exclusive_modifier(ExclusiveModifier::RequiresSpace),
                        "**",
                    );

                    let ctx = ctx.set_exclusive_modifier(ExclusiveModifier::Inline);
                    self.render_children(ctx, node.children());
                    self.render_text(ctx, "**");

                    RenderStatus::RenderedRequiresSpace
                }
                "em" => {
                    self.render_text(
                        ctx.merge_exclusive_modifier(ExclusiveModifier::RequiresSpace),
                        "_",
                    );

                    let ctx = ctx.set_exclusive_modifier(ExclusiveModifier::Inline);
                    self.render_children(ctx, node.children());
                    self.render_text(ctx, "_");

                    RenderStatus::RenderedRequiresSpace
                }
                "ul" => {
                    let mut status = RenderStatus::NotRendered;
                    for child in node.children() {
                        let st = self.render_node(
                            ctx.merge_exclusive_modifier(ExclusiveModifier::UnorderedList),
                            child,
                        );
                        if st.is_rendered() {
                            status = RenderStatus::Rendered;
                        }
                    }

                    status
                }
                "ol" => {
                    let mut status = RenderStatus::NotRendered;
                    let mut count = 1;
                    for child in node.children() {
                        let st = self.render_node(
                            ctx.merge_exclusive_modifier(ExclusiveModifier::OrderedList(count)),
                            child,
                        );
                        if st.is_rendered() {
                            status = RenderStatus::Rendered;
                            count += 1;
                        }
                    }

                    status
                }
                //     "h1" => render_header(&ctx, 1, node, res),
                //     "h2" => render_header(&ctx, 2, node, res),
                //     "h3" => render_header(&ctx, 3, node, res),
                //     "h4" => render_header(&ctx, 4, node, res),
                //     "h5" => render_header(&ctx, 5, node, res),
                //     "h6" => render_header(&ctx, 6, node, res),
                //     "code" => {
                //         let is_block = node.parent().is_some_and(|p| match p.value() {
                //             Node::Element(elt) => elt.name() == "pre",
                //             _ => false,
                //         });
                //
                //         if !is_block {
                //             render_context(&ctx.merge(ContextType::RequiresSpace), '`', res);
                //
                //             res.push('`');
                //             render_node(
                //                 node.children().next().unwrap(),
                //                 res,
                //                 Context {
                //                     context_type: ContextType::Inline,
                //                     indent: ctx.indent,
                //                     keep_prefix_space: ctx.keep_prefix_space,
                //                 },
                //             );
                //             res.push('`');
                //
                //             RenderStatus::RenderedRequiresSpace
                //         } else {
                //             render_context(&ctx, ' ', res);
                //             if matches!(
                //                 ctx.context_type,
                //                 ContextType::Inline | ContextType::RequiresSpace
                //             ) {
                //                 render_new_line(&ctx, res);
                //             }
                //
                //             res.push_str("```");
                //             for ch in node.children() {
                //                 render_new_line(&ctx, res);
                //
                //                 render_node(
                //                     ch,
                //                     res,
                //                     Context {
                //                         context_type: ContextType::Inline,
                //                         indent: ctx.indent,
                //                         keep_prefix_space: true,
                //                     },
                //                 );
                //             }
                //
                //             render_new_line(&ctx, res);
                //             res.push_str("```");
                //
                //             if matches!(
                //                 ctx.context_type,
                //                 ContextType::Inline | ContextType::RequiresSpace
                //             ) {
                //                 render_new_line(&ctx, res);
                //             }
                //
                //             RenderStatus::Rendered
                //         }
                //     }
                _ => {
                    let mut status = RenderStatus::NotRendered;
                    for child in node.children() {
                        let context = match status {
                            RenderStatus::NotRendered => {
                                ctx.merge_exclusive_modifier(ExclusiveModifier::NewParagraph)
                            }
                            RenderStatus::Rendered => {
                                ctx.set_exclusive_modifier(ExclusiveModifier::Inline)
                            }
                            RenderStatus::RenderedRequiresSpace => {
                                ctx.set_exclusive_modifier(ExclusiveModifier::RequiresSpace)
                            }
                        };

                        let st = self.render_node(context, child);
                        if st.is_rendered() {
                            status = st
                        }
                    }

                    if status.is_rendered() {
                        RenderStatus::Rendered
                    } else {
                        RenderStatus::NotRendered
                    }
                }
            },
            Node::Comment(_) => RenderStatus::NotRendered,
            Node::Doctype(_) => RenderStatus::NotRendered,
            Node::ProcessingInstruction(_) => RenderStatus::NotRendered,
        }
    }

    fn render_children(&mut self, ctx: Context, children: Children<'_, Node>) -> RenderStatus {
        let mut status = RenderStatus::NotRendered;

        for child in children {
            let st = self.render_node(ctx, child);
            if st.is_rendered() {
                status = st;
            }
        }

        status
    }

    fn render_text(&mut self, ctx: Context, text: &str) -> RenderStatus {
        let (prefix, txt) = if ctx.has_stackable_modifier(StackableModifier::KeepPrefixSpace) {
            let trimmed = text.trim_start();
            let trimmed_len = text.len() - trimmed.len();
            (&text[0..trimmed_len], trimmed)
        } else {
            ("", text.trim())
        };

        if prefix.is_empty() && txt.is_empty() {
            return RenderStatus::NotRendered;
        }

        let first_char = prefix.chars().next().or(txt.chars().next());
        self.render_context(ctx, first_char);

        if !prefix.is_empty() {
            self.lines.last_mut().unwrap().push_span(prefix.to_string());
            self.last_line_width += prefix.width();
        }

        // If we are rendering text inside a list (already indented), we have to increase indent
        // after going to new line. This way text is formatted nicely as:
        // ```text
        //   - first line
        //     second line
        // ```
        let increase_indent = ctx.indent > 0;

        let mut line_start = true;
        for word in txt.split_whitespace() {
            // Add + 1 for space
            if self.max_width < self.last_line_width + word.width() + 1 {
                self.render_new_line(ctx.indent, increase_indent);
                line_start = true;
            }

            let line = self.lines.last_mut().unwrap();
            if !line_start && self.last_line_width != 0 {
                line.push_span(" ");
                self.last_line_width += 1;
            }

            line.push_span(word.to_string());
            self.last_line_width += word.len();
            line_start = false;
        }

        RenderStatus::Rendered
    }

    fn render_context(&mut self, ctx: Context, first_char: Option<char>) {
        // TODO: Handle new lines at the beginning of the file

        match ctx.exclusive_modifier {
            ExclusiveModifier::Inline => (),
            ExclusiveModifier::RequiresSpace => {
                if first_char.is_none_or(|c| c != '.' && c != ',' && c != ';') {
                    self.lines.last_mut().unwrap().push_span(" ");
                    self.last_line_width += 1;
                }
            }
            ExclusiveModifier::NewParagraph => {
                self.render_new_line(ctx.indent, false);
                self.render_new_line(ctx.indent, false);
            }
            ExclusiveModifier::UnorderedList => {
                self.render_new_line(ctx.indent, false);
                self.lines.last_mut().unwrap().push_span("- ");
            }
            ExclusiveModifier::OrderedList(idx) => {
                self.render_new_line(ctx.indent, false);
                self.lines
                    .last_mut()
                    .unwrap()
                    .push_span(format!("{}. ", idx));
            }
        }
    }

    fn render_new_line(&mut self, indent: u16, increase_indent: bool) {
        self.lines.push(Line::default());

        let tabsize = 2;
        let indent_size = if increase_indent {
            (indent + 1) * tabsize
        } else {
            indent * tabsize
        };

        if indent_size > 0 {
            let mut ind = String::new();
            for _ in 0..indent_size {
                ind.push(' ');
            }
            self.lines.last_mut().unwrap().push_span(ind);
        }
        self.last_line_width = indent_size as usize;
    }
}

fn first_char(node: NodeRef<'_, Node>) -> Option<char> {
    match node.value() {
        Node::Document | Node::Fragment => node.first_child().and_then(first_char),
        Node::Text(text) => text.chars().next(),
        Node::Element(element) => match element.name() {
            "script" | "head" | "noscript" => None,
            "a" => Some('['),
            _ => node.first_child().and_then(first_char),
        },
        Node::Comment(_) => None,
        Node::Doctype(_) => None,
        Node::ProcessingInstruction(_) => None,
    }
}

// fn render_header(
//     ctx: &Context,
//     heading: u8,
//     node: NodeRef<'_, Node>,
//     res: &mut String,
// ) -> RenderStatus {
//     render_context(&ctx.merge(ContextType::NewParagraph), '#', res);
//
//     for _ in 0..heading {
//         res.push('#')
//     }
//
//     for ch in node.children() {
//         render_node(
//             ch,
//             res,
//             Context {
//                 context_type: ContextType::RequiresSpace,
//                 indent: ctx.indent,
//                 keep_prefix_space: ctx.keep_prefix_space,
//             },
//         );
//     }
//
//     RenderStatus::Rendered
// }

#[cfg(test)]
mod test {
    use super::render;

    #[test]
    fn simple() {
        let lines = render(r#"<p><a href="test">neki</a>. se neki</p>"#, 120);
        println!("{:?}", lines);
        assert_eq!(lines.len(), 1);
    }
}
