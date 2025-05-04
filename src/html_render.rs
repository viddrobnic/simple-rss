use ego_tree::{NodeRef, iter::Children};
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};
use scraper::{Html, Node};
use unicode_width::UnicodeWidthStr;

const TAB_SIZE: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackableModifier {
    InsideRawBlock = 1 << 0,
    InsideList = 1 << 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ExclusiveModifier {
    #[default]
    Inline,
    RequiresSpace,
    NewLine,
    NewParagraph,
    NewHeading,
    UnorderedList,
    OrderedList(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackableStyle {
    Bold = 1 << 0,
    Italic = 1 << 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ExclusiveStyle {
    #[default]
    Default,
    Code,
    Link,
    Heading,
}

#[derive(Debug, Clone, Copy, Default)]
struct Context {
    exclusive_modifier: ExclusiveModifier,
    stackable_modifiers: u8,

    exclusive_style: ExclusiveStyle,
    stackable_styles: u8,

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
            ExclusiveModifier::NewLine => 2,
            ExclusiveModifier::NewParagraph => 3,
            ExclusiveModifier::NewHeading => 4,
            ExclusiveModifier::UnorderedList => 5,
            ExclusiveModifier::OrderedList(_) => 5,
        }
    }
}

impl ExclusiveStyle {
    fn precedence(&self) -> u8 {
        match self {
            ExclusiveStyle::Default => 0,
            ExclusiveStyle::Code => 1,
            ExclusiveStyle::Link => 2,
            ExclusiveStyle::Heading => 3,
        }
    }
}

impl Context {
    fn merge_exclusive_modifier(mut self, modifier: ExclusiveModifier) -> Self {
        if self.exclusive_modifier.precedence() > modifier.precedence() {
            self
        } else {
            let indent = match modifier {
                ExclusiveModifier::UnorderedList | ExclusiveModifier::OrderedList(_) => {
                    self.indent + 1
                }
                _ => self.indent,
            };

            self.exclusive_modifier = modifier;
            self.indent = indent;
            self
        }
    }

    fn set_exclusive_modifier(mut self, modifier: ExclusiveModifier) -> Self {
        self.exclusive_modifier = modifier;
        self
    }

    fn has_stackable_modifier(&self, modifier: StackableModifier) -> bool {
        self.stackable_modifiers & modifier as u8 > 0
    }

    fn add_stackable_modifier(mut self, modifier: StackableModifier) -> Self {
        self.stackable_modifiers |= modifier as u8;
        self
    }

    fn remove_stackable_modifier(mut self, modifier: StackableModifier) -> Self {
        self.stackable_modifiers &= !(modifier as u8);
        self
    }

    fn merge_exclusive_style(mut self, style: ExclusiveStyle) -> Self {
        if self.exclusive_style.precedence() <= style.precedence() {
            self.exclusive_style = style;
        }

        self
    }

    fn set_exclusive_style(mut self, style: ExclusiveStyle) -> Self {
        self.exclusive_style = style;
        self
    }

    fn has_stackable_style(&self, style: StackableStyle) -> bool {
        self.stackable_styles & style as u8 > 0
    }

    fn add_stackable_style(mut self, style: StackableStyle) -> Self {
        self.stackable_styles |= style as u8;
        self
    }

    fn remove_stackable_style(mut self, style: StackableStyle) -> Self {
        self.stackable_styles &= !(style as u8);
        self
    }

    fn style(&self) -> Style {
        let mut style = match self.exclusive_style {
            ExclusiveStyle::Default => Style::default(),
            ExclusiveStyle::Code => Style::default().fg(Color::Gray),
            ExclusiveStyle::Link => Style::default().fg(Color::LightBlue),
            ExclusiveStyle::Heading => Style::default().fg(Color::Green).bold(),
        };

        if self.has_stackable_style(StackableStyle::Bold) {
            style = style.bold();
        }
        if self.has_stackable_style(StackableStyle::Italic) {
            style = style.italic();
        }

        style
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
                "span" | "button" => {
                    self.render_context(ctx, first_char(node));
                    self.render_children(
                        ctx.set_exclusive_modifier(ExclusiveModifier::Inline),
                        node.children(),
                    );

                    RenderStatus::RenderedRequiresSpace
                }
                "a" => {
                    let ctx = ctx.merge_exclusive_style(ExclusiveStyle::Link);
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
                    let ctx = ctx.add_stackable_style(StackableStyle::Bold);

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
                    let ctx = ctx.add_stackable_style(StackableStyle::Italic);
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
                    let ctx = ctx
                        .merge_exclusive_modifier(ExclusiveModifier::UnorderedList)
                        .add_stackable_modifier(StackableModifier::InsideList);

                    for child in node.children() {
                        let st = self.render_node(ctx, child);
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
                            ctx.merge_exclusive_modifier(ExclusiveModifier::OrderedList(count))
                                .add_stackable_modifier(StackableModifier::InsideList),
                            child,
                        );
                        if st.is_rendered() {
                            status = RenderStatus::Rendered;
                            count += 1;
                        }
                    }

                    status
                }
                "h1" => self.render_header(ctx, 1, node),
                "h2" => self.render_header(ctx, 2, node),
                "h3" => self.render_header(ctx, 3, node),
                "h4" => self.render_header(ctx, 4, node),
                "h5" => self.render_header(ctx, 5, node),
                "h6" => self.render_header(ctx, 6, node),
                "code" => {
                    let is_block = node.parent().is_some_and(|p| match p.value() {
                        Node::Element(elt) => elt.name() == "pre",
                        _ => false,
                    });

                    let ctx = ctx.merge_exclusive_style(ExclusiveStyle::Code);
                    if !is_block {
                        self.render_text(
                            ctx.merge_exclusive_modifier(ExclusiveModifier::RequiresSpace),
                            "`",
                        );

                        let ctx = ctx.set_exclusive_modifier(ExclusiveModifier::Inline);
                        self.render_children(ctx, node.children());
                        self.render_text(ctx, "`");

                        RenderStatus::RenderedRequiresSpace
                    } else {
                        self.render_text(
                            ctx.merge_exclusive_modifier(ExclusiveModifier::NewLine),
                            "```",
                        );

                        let context = ctx
                            .set_exclusive_modifier(ExclusiveModifier::Inline)
                            .add_stackable_modifier(StackableModifier::InsideRawBlock);

                        self.render_new_line(context);
                        for child in node.children() {
                            self.render_node(context, child);
                        }

                        self.render_text(
                            ctx.set_exclusive_modifier(ExclusiveModifier::NewLine),
                            "```",
                        );

                        if matches!(
                            ctx.exclusive_modifier,
                            ExclusiveModifier::Inline | ExclusiveModifier::RequiresSpace
                        ) {
                            self.render_new_line(ctx);
                        }

                        RenderStatus::Rendered
                    }
                }
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

    fn render_header(
        &mut self,
        ctx: Context,
        heading: u8,
        node: NodeRef<'_, Node>,
    ) -> RenderStatus {
        self.render_context(
            ctx.merge_exclusive_modifier(ExclusiveModifier::NewHeading),
            Some('#'),
        );

        let ctx = ctx.set_exclusive_style(ExclusiveStyle::Heading);
        for _ in 0..heading {
            self.render_text(ctx.set_exclusive_modifier(ExclusiveModifier::Inline), "#");
        }

        self.render_children(
            ctx.set_exclusive_modifier(ExclusiveModifier::RequiresSpace),
            node.children(),
        );

        RenderStatus::Rendered
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
        if ctx.has_stackable_modifier(StackableModifier::InsideRawBlock) {
            return self.render_raw_text(ctx, text);
        }

        let txt = text.trim();
        if txt.is_empty() {
            return RenderStatus::NotRendered;
        }

        let first_char = txt.chars().next();
        self.render_context(ctx, first_char);

        let style = ctx.style();

        let mut line_start = true;
        for word in txt.split_whitespace() {
            // Add + 1 for space
            if self.max_width < self.last_line_width + word.width() + 1 {
                self.render_new_line(ctx);
                line_start = true;
            }

            let line = self.lines.last_mut().unwrap();
            if !line_start && self.last_line_width != 0 {
                line.push_span(Span::from(" ").style(style));
                self.last_line_width += 1;
            }

            line.push_span(Span::from(word.to_string()).style(style));
            self.last_line_width += word.len();
            line_start = false;
        }

        RenderStatus::Rendered
    }

    fn render_raw_text(&mut self, ctx: Context, text: &str) -> RenderStatus {
        let style = ctx.style();

        for (idx, line) in text
            .replace('\r', "")
            .replace('\t', "    ")
            .lines()
            .enumerate()
        {
            if idx > 0 {
                self.render_new_line(ctx);
            }

            self.lines
                .last_mut()
                .unwrap()
                .push_span(Span::from(line.to_string()).style(style));
            self.last_line_width += line.width();
        }

        if text.ends_with('\n') {
            self.render_new_line(ctx);
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
            ExclusiveModifier::NewLine => {
                self.render_new_line(ctx);
            }
            ExclusiveModifier::NewParagraph => {
                self.render_new_line(ctx);
                self.render_new_line(ctx);
            }
            ExclusiveModifier::NewHeading => {
                self.render_new_line(ctx);
                self.render_new_line(ctx);
                self.render_new_line(ctx);
            }
            ExclusiveModifier::UnorderedList => {
                // We have to remove inside list modifier when rendering the first line of the
                // element.
                self.render_new_line(ctx.remove_stackable_modifier(StackableModifier::InsideList));
                self.lines
                    .last_mut()
                    .unwrap()
                    .push_span(Span::from("- ").style(Style::default().fg(Color::Gray)));
            }
            ExclusiveModifier::OrderedList(idx) => {
                self.render_new_line(ctx.remove_stackable_modifier(StackableModifier::InsideList));
                self.lines.last_mut().unwrap().push_span(
                    Span::from(format!("{}. ", idx)).style(Style::default().fg(Color::Gray)),
                );
            }
        }
    }

    fn render_new_line(&mut self, ctx: Context) {
        self.lines.push(Line::default());

        let indent = if ctx.has_stackable_modifier(StackableModifier::InsideList) {
            ctx.indent + 1
        } else {
            ctx.indent
        };

        let indent_size = indent * TAB_SIZE;

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

#[cfg(test)]
mod test {
    use super::render;

    #[test]
    fn simple() {
        let lines = render(
            r#"<pre><code class="language-protobuf" data-lang="protobuf"><span class="kn">package</span> <span class="nn">google</span><span class="o">.</span><span class="k">rpc</span><span class="p">;</span>
<span class="kd">enum</span> <span class="n">Code</span> <span class="p">{</span>
  <span class="na">OK</span> <span class="o">=</span> <span class="mi">0</span><span class="p">;</span>
  <span class="na">CANCELLED</span> <span class="o">=</span> <span class="mi">1</span><span class="p">;</span>
  <span class="na">UNKNOWN</span> <span class="o">=</span> <span class="mi">2</span><span class="p">;</span>
  <span class="c1">// ...</span>
<span class="p">}</span>

<span class="kn">package</span> <span class="nn">google</span><span class="o">.</span><span class="n">protobuf</span><span class="p">;</span>
<span class="kd">message</span> <span class="nc">FieldDescriptorProto</span> <span class="p">{</span>
  <span class="kd">enum</span> <span class="n">Type</span> <span class="p">{</span>
    <span class="c1">// 0 is reserved for errors.</span>
    <span class="na">TYPE_DOUBLE</span> <span class="o">=</span> <span class="mi">1</span><span class="p">;</span>
    <span class="na">TYPE_FLOAT</span> <span class="o">=</span> <span class="mi">2</span><span class="p">;</span>
    <span class="na">TYPE_INT64</span> <span class="o">=</span> <span class="mi">3</span><span class="p">;</span>
    <span class="c1">// ...</span>
  <span class="p">}</span>
<span class="p">}</span></code></pre>"#,
            120,
        );
        println!("{:?}", lines);
    }
}
