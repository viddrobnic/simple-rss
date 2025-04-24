use ego_tree::{NodeRef, iter::Children};
use scraper::{Html, Node};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextType {
    Inline,
    RequiresSpace,
    NewParagraph,
    UnorderedList,
    OrderedList(u16),
}

#[derive(Debug, Clone, Copy)]
struct Context {
    context_type: ContextType,
    indent: u16,
    keep_prefix_space: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderStatus {
    NotRendered,
    Rendered,
    RenderedRequiresSpace,
}

impl ContextType {
    fn precedence(&self) -> u8 {
        match self {
            ContextType::Inline => 0,
            ContextType::RequiresSpace => 1,
            ContextType::NewParagraph => 2,
            ContextType::UnorderedList => 3,
            ContextType::OrderedList(_) => 3,
        }
    }
}

impl Context {
    fn merge(&self, context_type: ContextType) -> Self {
        if self.context_type.precedence() > context_type.precedence() {
            Context {
                context_type: self.context_type,
                keep_prefix_space: self.keep_prefix_space,
                indent: self.indent,
            }
        } else {
            let indent = match context_type {
                ContextType::UnorderedList | ContextType::OrderedList(_) => self.indent + 2,
                _ => self.indent,
            };

            Context {
                context_type,
                keep_prefix_space: self.keep_prefix_space,
                indent,
            }
        }
    }
}

impl RenderStatus {
    fn is_rendered(&self) -> bool {
        *self != RenderStatus::NotRendered
    }
}

pub fn render(html: &str) {
    let tree = Html::parse_document(html);

    let root = tree.tree.root();
    let mut res = String::new();
    render_node(
        root,
        &mut res,
        Context {
            context_type: ContextType::Inline,
            indent: 0,
            keep_prefix_space: false,
        },
    );

    println!("{}", res);
}

fn render_node(node: NodeRef<'_, Node>, res: &mut String, ctx: Context) -> RenderStatus {
    match node.value() {
        Node::Document => render_children(node.children(), res, ctx),
        Node::Fragment => render_children(node.children(), res, ctx),
        Node::Text(text) => {
            let txt = if ctx.keep_prefix_space {
                &text.text
            } else {
                text.text.trim()
            };

            if txt.is_empty() {
                return RenderStatus::NotRendered;
            }

            let txt = txt.replace("\n", " ");

            render_context(&ctx, txt.chars().next().unwrap(), res);
            res.push_str(&txt);

            RenderStatus::Rendered
        }
        Node::Element(element) => match element.name() {
            "a" => {
                render_context(&ctx.merge(ContextType::RequiresSpace), '[', res);

                res.push('[');
                render_children(
                    node.children(),
                    res,
                    Context {
                        context_type: ContextType::Inline,
                        indent: ctx.indent,
                        keep_prefix_space: ctx.keep_prefix_space,
                    },
                );
                res.push_str("](");
                res.push_str(element.attr("href").unwrap_or(""));
                res.push(')');

                RenderStatus::RenderedRequiresSpace
            }
            "span" => {
                render_context(&ctx, ' ', res);
                render_children(
                    node.children(),
                    res,
                    Context {
                        context_type: ContextType::Inline,
                        indent: ctx.indent,
                        keep_prefix_space: ctx.keep_prefix_space,
                    },
                );

                RenderStatus::RenderedRequiresSpace
            }
            "strong" => {
                render_context(&ctx, ' ', res);

                res.push_str("**");
                render_children(
                    node.children(),
                    res,
                    Context {
                        context_type: ContextType::Inline,
                        indent: ctx.indent,
                        keep_prefix_space: ctx.keep_prefix_space,
                    },
                );
                res.push_str("**");

                RenderStatus::RenderedRequiresSpace
            }
            "script" => RenderStatus::NotRendered, // ignore
            "ul" => {
                let mut status = RenderStatus::NotRendered;
                for ch in node.children() {
                    let st = render_node(ch, res, ctx.merge(ContextType::UnorderedList));
                    if st.is_rendered() {
                        status = RenderStatus::Rendered;
                    }
                }

                status
            }
            "ol" => {
                let mut status = RenderStatus::NotRendered;
                let mut count = 1;
                for ch in node.children() {
                    let st = render_node(ch, res, ctx.merge(ContextType::OrderedList(count)));
                    if st.is_rendered() {
                        status = RenderStatus::Rendered;
                        count += 1;
                    }
                }

                status
            }
            "code" => {
                let is_block = node.parent().is_some_and(|p| match p.value() {
                    Node::Element(elt) => elt.name() == "pre",
                    _ => false,
                });

                if !is_block {
                    render_context(&ctx.merge(ContextType::RequiresSpace), '`', res);

                    res.push('`');
                    render_node(
                        node.children().next().unwrap(),
                        res,
                        Context {
                            context_type: ContextType::Inline,
                            indent: ctx.indent,
                            keep_prefix_space: ctx.keep_prefix_space,
                        },
                    );
                    res.push('`');

                    RenderStatus::RenderedRequiresSpace
                } else {
                    render_context(&ctx, ' ', res);
                    if matches!(
                        ctx.context_type,
                        ContextType::Inline | ContextType::RequiresSpace
                    ) {
                        new_line(&ctx, res);
                    }

                    res.push_str("```");
                    for ch in node.children() {
                        new_line(&ctx, res);

                        render_node(
                            ch,
                            res,
                            Context {
                                context_type: ContextType::Inline,
                                indent: ctx.indent,
                                keep_prefix_space: true,
                            },
                        );
                    }

                    new_line(&ctx, res);
                    res.push_str("```");

                    if matches!(
                        ctx.context_type,
                        ContextType::Inline | ContextType::RequiresSpace
                    ) {
                        new_line(&ctx, res);
                    }

                    RenderStatus::Rendered
                }
            }
            _ => {
                let mut status = RenderStatus::NotRendered;
                for ch in node.children() {
                    let context = match status {
                        RenderStatus::NotRendered => ctx.merge(ContextType::NewParagraph),
                        RenderStatus::Rendered => Context {
                            context_type: ContextType::Inline,
                            indent: ctx.indent,
                            keep_prefix_space: ctx.keep_prefix_space,
                        },
                        RenderStatus::RenderedRequiresSpace => Context {
                            context_type: ContextType::RequiresSpace,
                            indent: ctx.indent,
                            keep_prefix_space: ctx.keep_prefix_space,
                        },
                    };

                    let st = render_node(ch, res, context);
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

fn render_children(children: Children<'_, Node>, res: &mut String, ctx: Context) -> RenderStatus {
    let mut status = RenderStatus::NotRendered;

    for ch in children {
        let st = render_node(ch, res, ctx);
        if st.is_rendered() {
            status = st;
        }
    }

    status
}

fn new_line(ctx: &Context, res: &mut String) {
    res.push('\n');
    for _ in 0..ctx.indent {
        res.push(' ');
    }
}

fn render_context(ctx: &Context, first_char: char, res: &mut String) {
    match ctx.context_type {
        ContextType::Inline => (),
        ContextType::RequiresSpace => {
            if first_char != '.' && first_char != ',' && first_char != ';' {
                res.push(' ');
            }
        }
        ContextType::NewParagraph => {
            new_line(ctx, res);
            new_line(ctx, res);
        }
        ContextType::UnorderedList => {
            new_line(ctx, res);
            res.push_str("- ");
        }
        ContextType::OrderedList(idx) => {
            new_line(ctx, res);
            res.push_str(&format!("{}. ", idx));
        }
    }
}

#[cfg(test)]
mod test {
    use super::render;

    #[test]
    fn asdf() {
        let html = r##"
<!DOCTYPE html>
<html lang="en-us"
      dir="ltr">
    <head><meta charset="utf-8">
<meta name="viewport" content="width=device-width">
<title>Writing My Own Language (Part 2) | Vid Drobnič</title>

<link rel="stylesheet" href="/css/main.css">



<script>
function onThemeChange() {
    if (localStorage.theme === 'dark'){
        document.documentElement.classList.add('dark')
    } else {
        document.documentElement.classList.remove('dark')
    }
}

onThemeChange()
</script>
</head>
    <body class="bg-white transition-colors dark:bg-slate-800">
        <div class="flex flex-col px-6 pt-10 pb-10 mx-auto space-y-12 max-w-5xl sm:flex-row sm:justify-center sm:space-y-0 sm:space-x-8">
            <nav class="flex flex-col self-start space-y-2 sm:sticky sm:top-10 sm:space-y-4 dark:text-white w-fit">
                <a class="mb-2 text-2xl font-medium no-underline sm:mb-0 text-nowrap"
                   href="/">Vid Drobnič</a>
                <div class="flex flex-row space-x-2 sm:flex-col sm:space-y-1 sm:space-x-0">
                    
                    
                    <a href="/"
                       class="">About</a>
                    
                    
                    <a href="/blog/"
                       class="font-semibold no-underline">Blog</a>
                    
                    
                    <a href="/projects/"
                       class="">Projects</a>
                    
                    
                    <a href="/misc/"
                       class="">Misc</a>
                    
                    
                    <a href="https://github.com/viddrobnic"
                       class="">GitHub</a>
                    
                </div>
                <div class="relative">
                    <div class="flex absolute items-center dark:hidden">
                        <a class="text-sm font-light cursor-pointer" onclick="setTheme('dark')">Dark Mode</a>
                        <svg xmlns="http://www.w3.org/2000/svg"
                             width="24"
                             height="24"
                             viewBox="0 0 24 24"
                             fill="none"
                             stroke="currentColor"
                             stroke-width="2"
                             stroke-linecap="round"
                             stroke-linejoin="round"
                             class="ml-1 w-4 h-4">
                            <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
                        </svg>
                    </div>
                    <div class="hidden absolute items-center dark:flex">
                        <a class="text-sm font-light cursor-pointer" onclick="setTheme('light')">Light Mode</a>
                        <svg xmlns="http://www.w3.org/2000/svg"
                             width="24"
                             height="24"
                             viewBox="0 0 24 24"
                             fill="none"
                             stroke="currentColor"
                             stroke-width="2"
                             stroke-linecap="round"
                             stroke-linejoin="round"
                             class="ml-1 w-4 h-4">
                            <circle cx="12" cy="12" r="4" />
                            <path d="M12 2v2" />
                            <path d="M12 20v2" />
                            <path d="m4.93 4.93 1.41 1.41" />
                            <path d="m17.66 17.66 1.41 1.41" />
                            <path d="M2 12h2" />
                            <path d="M20 12h2" />
                            <path d="m6.34 17.66-1.41 1.41" />
                            <path d="m19.07 4.93-1.41 1.41" />
                        </svg>
                    </div>
                </div>
            </nav>
            <div class="w-full prose dark:prose-invert">
                <h1>Writing My Own Language (Part 2)</h1>
                
<div class="flex flex-wrap justify-between -mt-6 prose prose-p:my-0 dark:prose-invert">
    <p class="mr-8 font-light">25 Jul 2024</p>
    <p class="font-light">7 minute read</p>
</div>
<div class="py-3 px-2 mt-6 bg-gray-100 rounded-xl shadow-md prose prose-h3:mt-0 prose-h3:mb-1 prose-li:my-0 prose-ul:my-0 prose-ul:list-none prose-ul:ps-0 prose-li:ps-4 prose-a:no-underline prose-a:font-normal dark:bg-slate-700 dark:prose-invert">
    <h3 class="pl-3.5">Contents</h3>
    <nav id="TableOfContents">
  <ul>
    <li><a href="#approach">Approach</a></li>
    <li><a href="#defining-the-grammar">Defining The Grammar</a>
      <ul>
        <li><a href="#basic-structure">Basic Structure</a></li>
        <li><a href="#prefix--infix">Prefix &amp; Infix</a></li>
        <li><a href="#blocks">Blocks</a></li>
        <li><a href="#strings">Strings</a></li>
      </ul>
    </li>
    <li><a href="#highlighting-queries">Highlighting Queries</a></li>
    <li><a href="#conclusion">Conclusion</a></li>
  </ul>
</nav>
</div>
<p>I decided to write <a href="https://github.com/viddrobnic/aoc-lang">my own programming language</a>
to solve this year&rsquo;s <a href="https://adventofcode.com/">Advent of Code</a>.
This includes an interpreter, syntax highlighting, and a language server. In the
<a href="https://viddrobnic.com/blog/2024/writing-my-language/">first part</a>, we looked at how the
interpreter works. In this part, we&rsquo;ll take a look at how I
implemented syntax highlighting.</p>
<h2 id="approach">
    <a href="#approach" class="no-underline peer" style="font-weight: inherit;">
        Approach
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h2>
<p>How to implement syntax highlighting depends on the editor that you use. I use
<a href="https://neovim.io/">Neovim</a>, where
<a href="https://tree-sitter.github.io/tree-sitter/">Tree-sitter</a> is
<a href="https://github.com/nvim-treesitter/nvim-treesitter">usually used</a> for it.</p>
<p>Tree-sitter is a fast and robust parser generator tool. It can be used to
(relatively) easily create a parser for a language. The generated parser can
be used to walk and inspect the syntax tree. It also supports advanced
<a href="https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries">pattern matching with queries</a>.</p>
<p>I implemented the syntax highlighting using the following steps:</p>
<ol>
<li><a href="https://tree-sitter.github.io/tree-sitter/creating-parsers">Create a parser</a>
for the AOC language by defining its grammar.</li>
<li>Define which nodes in the syntax tree represent keywords, variables, values, etc.
This is done by defining <a href="https://github.com/nvim-treesitter/nvim-treesitter/blob/master/CONTRIBUTING.md#highlights">highlighting queries</a>
using the previously mentioned query language.</li>
<li>Update the Neovim configuration to use a custom Tree-sitter grammar
and highlighting queries.</li>
</ol>
<h2 id="defining-the-grammar">
    <a href="#defining-the-grammar" class="no-underline peer" style="font-weight: inherit;">
        Defining The Grammar
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h2>
<p>Tree-sitter rules are defined in the <code>grammar.js</code> file. Each rule has a name
and can be defined as a string constant, regular expression, or a combination
of other rules by using the provided functions. Rules whose names are prefixed
with <code>'_'</code> are hidden in the syntax tree. This is useful for defining rules
like <code>_expression</code>, which wraps just one node. For those interested, a more
in-depth explanation of how to write rules is available in
<a href="https://tree-sitter.github.io/tree-sitter/creating-parsers">the official documentation</a>.</p>
<h3 id="basic-structure">
    <a href="#basic-structure" class="no-underline peer" style="font-weight: inherit;">
        Basic Structure
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h3>
<p>All the possible rules are collected in the <code>_rules</code> rule, which is defined as:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">_rules: ($) =&gt;
</span></span><span class="line"><span class="cl">  choice(
</span></span><span class="line"><span class="cl">    $._expression,
</span></span><span class="line"><span class="cl">    $.assignment,
</span></span><span class="line"><span class="cl">    $.for_loop,
</span></span><span class="line"><span class="cl">    $.while_loop,
</span></span><span class="line"><span class="cl">    $.continue,
</span></span><span class="line"><span class="cl">    $.break,
</span></span><span class="line"><span class="cl">    $.return,
</span></span><span class="line"><span class="cl">  );
</span></span></code></pre></div><p>The function <code>choice</code> matches one of the possible rules. This allows me to
reference any rule later in the grammar. As you can see, I also grouped all
expressions<sup id="fnref:1"><a href="#fn:1" class="footnote-ref" role="doc-noteref">1</a></sup> into an <code>_expression</code> rule.</p>
<p>The root node of the syntax tree is the <code>source_file</code> node, which is defined like this:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="nx">source_file</span><span class="o">:</span> <span class="p">(</span><span class="nx">$</span><span class="p">)</span> <span class="p">=&gt;</span> <span class="nx">repeat</span><span class="p">(</span><span class="nx">seq</span><span class="p">(</span><span class="nx">$</span><span class="p">.</span><span class="nx">_rules</span><span class="p">,</span> <span class="nx">terminator</span><span class="p">)),</span>
</span></span></code></pre></div><p>where <code>terminator</code> is</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="kr">const</span> <span class="nx">terminator</span> <span class="o">=</span> <span class="nx">choice</span><span class="p">(</span><span class="s2">&#34;\n&#34;</span><span class="p">,</span> <span class="s2">&#34;\0&#34;</span><span class="p">);</span>
</span></span></code></pre></div><p>This basically means that the <code>source_file</code> is a sequence of many rules and each
rule ends with a new line or EOF.</p>
<p>The mandatory new line character between the rules is handled here and not in
the <code>_rules</code> node, because I sometimes need to reference the <code>_rules</code> node
and don&rsquo;t need it to be terminated with a new line character.
Take <code>for</code> loops as an example:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">for (initial; condition; after) { ... }
</span></span></code></pre></div><p>Here, <code>initial</code> and <code>after</code> are both <code>_rules</code> nodes but don&rsquo;t require
ending with a new line character.</p>
<p>With the basic grammar structure laid down, we can take a look at some more
interesting rules.</p>
<h3 id="prefix--infix">
    <a href="#prefix--infix" class="no-underline peer" style="font-weight: inherit;">
        Prefix &amp; Infix
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h3>
<p>For prefix and infix expressions, I obviously had to deal with precedence. It
turns out that Tree-sitter makes this easy for us. First, we define a <code>PREC</code>
enum, which is just copied over from the interpreter code.</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="kr">const</span> <span class="nx">PREC</span> <span class="o">=</span> <span class="p">{</span>
</span></span><span class="line"><span class="cl">  <span class="nx">lowest</span><span class="o">:</span> <span class="mi">0</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">assign</span><span class="o">:</span> <span class="mi">1</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">or</span><span class="o">:</span> <span class="mi">2</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">and</span><span class="o">:</span> <span class="mi">3</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">equals</span><span class="o">:</span> <span class="mi">4</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">less_greater</span><span class="o">:</span> <span class="mi">5</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">sum</span><span class="o">:</span> <span class="mi">6</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">product</span><span class="o">:</span> <span class="mi">7</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">prefix</span><span class="o">:</span> <span class="mi">8</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">  <span class="nx">call_index</span><span class="o">:</span> <span class="mi">9</span><span class="p">,</span>
</span></span><span class="line"><span class="cl"><span class="p">};</span>
</span></span></code></pre></div><p>Then we can use the <code>prec.left</code> function to define the left precedence of a rule.
We use <code>prec.left</code> instead of <code>prec</code> because the interpreter is also
left-associative and we want to keep that behavior.</p>
<p>A prefix expression is easy since there are only two prefix operators and both
have the same precedence.</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="nx">prefix_expression</span><span class="o">:</span> <span class="p">(</span><span class="nx">$</span><span class="p">)</span> <span class="p">=&gt;</span>
</span></span><span class="line"><span class="cl">   <span class="nx">prec</span><span class="p">.</span><span class="nx">left</span><span class="p">(</span>
</span></span><span class="line"><span class="cl">     <span class="nx">PREC</span><span class="p">.</span><span class="nx">prefix</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">     <span class="nx">seq</span><span class="p">(</span><span class="nx">field</span><span class="p">(</span><span class="s2">&#34;operator&#34;</span><span class="p">,</span> <span class="nx">choice</span><span class="p">(</span><span class="s2">&#34;!&#34;</span><span class="p">,</span> <span class="s2">&#34;-&#34;</span><span class="p">)),</span> <span class="nx">field</span><span class="p">(</span><span class="s2">&#34;right&#34;</span><span class="p">,</span> <span class="nx">$</span><span class="p">.</span><span class="nx">_expression</span><span class="p">)),</span>
</span></span><span class="line"><span class="cl">   <span class="p">),</span>
</span></span></code></pre></div><p>Function <code>field</code> creates a named field in the node, that can later be used
for more exact querying.</p>
<p>There are many infix expressions, and they have different precedences.
I used JS&rsquo;s <code>map</code> function to make it simple:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="nx">infix_expression</span><span class="o">:</span> <span class="p">(</span><span class="nx">$</span><span class="p">)</span> <span class="p">=&gt;</span> <span class="p">{</span>
</span></span><span class="line"><span class="cl">   <span class="kr">const</span> <span class="nx">operators</span> <span class="o">=</span> <span class="p">[</span>
</span></span><span class="line"><span class="cl">     <span class="p">[</span><span class="s2">&#34;+&#34;</span><span class="p">,</span> <span class="nx">PREC</span><span class="p">.</span><span class="nx">sum</span><span class="p">],</span>
</span></span><span class="line"><span class="cl">     <span class="p">[</span><span class="s2">&#34;*&#34;</span><span class="p">,</span> <span class="nx">PREC</span><span class="p">.</span><span class="nx">product</span><span class="p">],</span>
</span></span><span class="line"><span class="cl">     <span class="c1">// ...
</span></span></span><span class="line"><span class="cl"><span class="c1"></span>   <span class="p">];</span>
</span></span><span class="line"><span class="cl">
</span></span><span class="line"><span class="cl">   <span class="k">return</span> <span class="nx">choice</span><span class="p">(</span>
</span></span><span class="line"><span class="cl">     <span class="p">...</span><span class="nx">operators</span><span class="p">.</span><span class="nx">map</span><span class="p">(([</span><span class="nx">operator</span><span class="p">,</span> <span class="nx">precedence</span><span class="p">])</span> <span class="p">=&gt;</span>
</span></span><span class="line"><span class="cl">       <span class="nx">prec</span><span class="p">.</span><span class="nx">left</span><span class="p">(</span>
</span></span><span class="line"><span class="cl">         <span class="nx">precedence</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">         <span class="nx">seq</span><span class="p">(</span>
</span></span><span class="line"><span class="cl">           <span class="nx">field</span><span class="p">(</span><span class="s2">&#34;left&#34;</span><span class="p">,</span> <span class="nx">$</span><span class="p">.</span><span class="nx">_expression</span><span class="p">),</span>
</span></span><span class="line"><span class="cl">           <span class="nx">field</span><span class="p">(</span><span class="s2">&#34;operator&#34;</span><span class="p">,</span> <span class="nx">operator</span><span class="p">),</span>
</span></span><span class="line"><span class="cl">           <span class="nx">field</span><span class="p">(</span><span class="s2">&#34;right&#34;</span><span class="p">,</span> <span class="nx">$</span><span class="p">.</span><span class="nx">_expression</span><span class="p">),</span>
</span></span><span class="line"><span class="cl">         <span class="p">),</span>
</span></span><span class="line"><span class="cl">       <span class="p">),</span>
</span></span><span class="line"><span class="cl">     <span class="p">),</span>
</span></span><span class="line"><span class="cl">   <span class="p">);</span>
</span></span><span class="line"><span class="cl"><span class="p">},</span>
</span></span></code></pre></div><h3 id="blocks">
    <a href="#blocks" class="no-underline peer" style="font-weight: inherit;">
        Blocks
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h3>
<p>Remember how the <code>source_file</code> rule also handles the required new line character?
There is one more place we have to handle the new line character: block expressions.
A block expression contains whatever is between the <code>{</code> and <code>}</code> in function
definitions, for loops, if statements, etc.</p>
<p>However, the block expression is different from <code>source_file</code> in one way:
the last statement doesn&rsquo;t have to end with a new line. For instance, we can write:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">if (true) { 10 }
</span></span></code></pre></div><p>I had to handle an empty block separately; otherwise, the grammar was ambiguous
and Tree-sitter couldn&rsquo;t generate the parser from it. In the end, the
<code>block</code> definition looks like this:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="nx">block</span><span class="o">:</span> <span class="p">(</span><span class="nx">$</span><span class="p">)</span> <span class="p">=&gt;</span>
</span></span><span class="line"><span class="cl">   <span class="nx">choice</span><span class="p">(</span>
</span></span><span class="line"><span class="cl">     <span class="c1">// empty block
</span></span></span><span class="line"><span class="cl"><span class="c1"></span>     <span class="nx">seq</span><span class="p">(</span><span class="s2">&#34;{&#34;</span><span class="p">,</span> <span class="s2">&#34;}&#34;</span><span class="p">),</span>
</span></span><span class="line"><span class="cl">
</span></span><span class="line"><span class="cl">     <span class="c1">// at least one rule
</span></span></span><span class="line"><span class="cl"><span class="c1"></span>     <span class="nx">seq</span><span class="p">(</span>
</span></span><span class="line"><span class="cl">       <span class="s2">&#34;{&#34;</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">       <span class="nx">repeat</span><span class="p">(</span><span class="nx">seq</span><span class="p">(</span><span class="nx">$</span><span class="p">.</span><span class="nx">_rules</span><span class="p">,</span> <span class="nx">terminator</span><span class="p">)),</span>
</span></span><span class="line"><span class="cl">       <span class="nx">seq</span><span class="p">(</span><span class="nx">$</span><span class="p">.</span><span class="nx">_rules</span><span class="p">,</span> <span class="nx">optional</span><span class="p">(</span><span class="nx">terminator</span><span class="p">)),</span>
</span></span><span class="line"><span class="cl">       <span class="s2">&#34;}&#34;</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">     <span class="p">),</span>
</span></span><span class="line"><span class="cl">   <span class="p">),</span>
</span></span></code></pre></div><p>I used a similar trick for arrays, dictionaries, function definitions,
and function calls, where the last expression can be followed by a <code>,</code> or not.</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">// Both definitions are valid
</span></span><span class="line"><span class="cl">a = [1, 2]
</span></span><span class="line"><span class="cl">b = [1, 2,]
</span></span></code></pre></div><h3 id="strings">
    <a href="#strings" class="no-underline peer" style="font-weight: inherit;">
        Strings
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h3>
<p>Correctly parsing strings turns out to be more challenging than it looks.
This is due to the fact that our strings also support escaping,
which means that this is a valid string:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">&#34;foo \&#34;bar\&#34;&#34;
</span></span></code></pre></div><p>Therefore, we can&rsquo;t just write a regular expression that captures everything
between two <code>&quot;</code>.</p>
<p>AOC lang&rsquo;s strings are very similar to <a href="https://go.dev">Go</a>&rsquo;s strings, so I took
inspiration from <a href="https://github.com/tree-sitter/tree-sitter-go">tree-sitter-go</a>.</p>
<p>To define a <code>string</code> rule, we have to use <code>token.immediate</code>.
The official documentation tells us the following about it:</p>
<blockquote>
<p>Usually, whitespace (and any other extras, such as comments) is optional before
each token. This function means that the token will only match if there is no whitespace.</p></blockquote>
<p>With this newly learned function, we can define a <code>string</code> as a sequence of
alternating basic contents and escape sequences:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="nx">string</span><span class="o">:</span> <span class="p">(</span><span class="nx">$</span><span class="p">)</span> <span class="p">=&gt;</span>
</span></span><span class="line"><span class="cl">   <span class="nx">seq</span><span class="p">(</span>
</span></span><span class="line"><span class="cl">     <span class="s1">&#39;&#34;&#39;</span><span class="p">,</span>
</span></span><span class="line"><span class="cl">     <span class="nx">repeat</span><span class="p">(</span><span class="nx">choice</span><span class="p">(</span><span class="nx">$</span><span class="p">.</span><span class="nx">_string_basic_content</span><span class="p">,</span> <span class="nx">$</span><span class="p">.</span><span class="nx">escape_sequence</span><span class="p">)),</span>
</span></span><span class="line"><span class="cl">     <span class="nx">token</span><span class="p">.</span><span class="nx">immediate</span><span class="p">(</span><span class="s1">&#39;&#34;&#39;</span><span class="p">),</span>
</span></span><span class="line"><span class="cl">   <span class="p">),</span>
</span></span></code></pre></div><p>The basic content of a string is anything that is not <code>&quot;</code>, <code>\n</code>, or <code>\</code>.
Or, in the language of regular expressions, the basic content is <code>[^&quot;\n\\]+</code>.
In the grammar, I defined it as:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="nx">_string_basic_content</span><span class="o">:</span> <span class="p">()</span> <span class="p">=&gt;</span> <span class="nx">token</span><span class="p">.</span><span class="nx">immediate</span><span class="p">(</span><span class="nx">prec</span><span class="p">(</span><span class="mi">1</span><span class="p">,</span> <span class="sr">/[^&#34;\n\\]+/</span><span class="p">)),</span>
</span></span></code></pre></div><p>We use <code>token.immediate</code> to make sure that new lines and comments are handled
by string rules and not ignored. We also give it a precedence of 1 to ensure
that the grammar is not ambiguous and that something is parsed as an escape
sequence only if it&rsquo;s not basic content.</p>
<p>While on the topic of escape sequences, let&rsquo;s define them! They are easier to
define than basic content, since an escape sequence is just <code>\</code> followed
by a single character:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-js" data-lang="js"><span class="line"><span class="cl"><span class="nx">escape_sequence</span><span class="o">:</span> <span class="p">()</span> <span class="p">=&gt;</span> <span class="nx">token</span><span class="p">.</span><span class="nx">immediate</span><span class="p">(</span><span class="sr">/\\./</span><span class="p">),</span>
</span></span></code></pre></div><h2 id="highlighting-queries">
    <a href="#highlighting-queries" class="no-underline peer" style="font-weight: inherit;">
        Highlighting Queries
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h2>
<p>After the grammar was complete, I could add it to my neovim config<sup id="fnref:2"><a href="#fn:2" class="footnote-ref" role="doc-noteref">2</a></sup> and
start working on highlighting queries.</p>
<p>Most of the highlighting queries were pretty straightforward. I looked at the
<a href="https://github.com/nvim-treesitter/nvim-treesitter/blob/master/CONTRIBUTING.md#highlights">nvim-treesitter highlighting documentation</a>
and wrote queries for the capture groups that I wanted to implement. For instance,
here is an example of how I implemented various <code>@keyword</code> capture sub-groups:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">(continue) @keyword
</span></span><span class="line"><span class="cl">(break) @keyword
</span></span><span class="line"><span class="cl">&#34;fn&#34; @keyword.function
</span></span><span class="line"><span class="cl">&#34;use&#34; @keyword.import
</span></span><span class="line"><span class="cl">[&#34;for&#34; &#34;while&#34;] @keyword.repeat
</span></span><span class="line"><span class="cl">&#34;return&#34; @keyword.return
</span></span><span class="line"><span class="cl">[&#34;if&#34; &#34;else&#34;] @keyword.conditional
</span></span></code></pre></div><p>However, there were some things that I had to change in the grammar definition
to allow for correct querying.</p>
<p>First, I learned that adding fields on nodes allows for more precise querying.
For instance, I initially didn&rsquo;t have a <code>function</code> field on the <code>function_call</code>
node. The initial query for the <code>@function.call</code> group therefore looked like this:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">(function_call
</span></span><span class="line"><span class="cl">  (identifier) @function.call)
</span></span></code></pre></div><p>But this highlighted all the identifiers in the function call as if they were
a <code>@function.call</code>. In this example:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">foo(a, b, c)
</span></span></code></pre></div><p><code>a</code>, <code>b</code>, <code>c</code> are all identifiers and get captured as the <code>@function.call</code>
group. But the correct query would capture only <code>foo</code> as <code>@function.call</code>.</p>
<p>After adding the <code>function</code> field to the <code>function_call</code> node, the query was
easy to write and worked as intended:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">(function_call
</span></span><span class="line"><span class="cl">  function: (identifier) @function.call)
</span></span></code></pre></div><p>The second change I had to implement in my grammar definition was to define
two different nodes for the two different ways of indexing a dictionary.
If you remember from the <a href="https://viddrobnic.com/blog/2024/writing-my-language/#objects">first part</a>,
there are two ways to index a dictionary:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">foo[&#34;bar&#34;]
</span></span><span class="line"><span class="cl">foo.bar
</span></span></code></pre></div><p>If a user indexes an object with dot notation, I want to highlight the index
as a <code>@property</code>. Having a separate <code>dot_index</code> node for the dot index made this easy:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">(dot_index
</span></span><span class="line"><span class="cl">  index: (identifier) @property)
</span></span></code></pre></div><p>With the highlighting queries complete, the code now looks like this:
<figure class="figure-center"><img src="/blog/2024/writing-my-language-2/result.png" width="500">
</figure>
</p>
<h2 id="conclusion">
    <a href="#conclusion" class="no-underline peer" style="font-weight: inherit;">
        Conclusion
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h2>
<p>I was positively surprised by how easy it is to get started with tree-sitter.
There is a ton of documentation and examples on how to write a parser.
And getting started with writing queries is even easier! Overall I enjoyed
using tree-sitter and hope to be able to use it in the future as well.</p>
<div class="footnotes" role="doc-endnotes">
<hr>
<ol>
<li id="fn:1">
<p>In the <a href="https://viddrobnic.com/blog/2024/writing-my-language/">first part</a>, I
defined an expression to be anything that &ldquo;returns a value.&rdquo; For instance,
identifiers, infix operations, and array literals are all expressions.&#160;<a href="#fnref:1" class="footnote-backref" role="doc-backlink">&#x21a9;&#xfe0e;</a></p>
</li>
<li id="fn:2">
<p>If you want to add AOC lang highlighting to your neovim config, see
<a href="https://github.com/viddrobnic/aoc-lang?tab=readme-ov-file#syntax-highlighting">the project&rsquo;s README</a>.&#160;<a href="#fnref:2" class="footnote-backref" role="doc-backlink">&#x21a9;&#xfe0e;</a></p>
</li>
</ol>
</div>


            </div>
        </div>
        
        <script>
        function setTheme(theme){
            localStorage.theme = theme
            onThemeChange()
        }
        </script>
        
        
        <script defer
                data-domain="viddrobnic.com"
                src="https://a.viddrobnic.com/js/script.js"></script>
        
    </body>
</html>

    "##;

        render(html);
    }
}
