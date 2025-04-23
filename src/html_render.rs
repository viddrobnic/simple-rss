use anyhow::bail;
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
        let ctx_type = if self.context_type.precedence() > context_type.precedence() {
            self.context_type
        } else {
            context_type
        };

        Self {
            context_type: ctx_type,
            indent: self.indent,
        }
    }
}

impl RenderStatus {
    fn is_rendered(&self) -> bool {
        *self != RenderStatus::NotRendered
    }
}

pub fn render(html: &str) -> anyhow::Result<()> {
    let tree = Html::parse_document(html);
    if !tree.errors.is_empty() {
        bail!("invalid html: {:?}", tree.errors);
    }

    let root = tree.tree.root();
    let mut res = String::new();
    render_node(
        root,
        &mut res,
        Context {
            context_type: ContextType::Inline,
            indent: 0,
        },
    );

    println!("{}", res);

    Ok(())
}

fn render_node(node: NodeRef<'_, Node>, res: &mut String, ctx: Context) -> RenderStatus {
    match node.value() {
        Node::Document => render_children(node.children(), res, ctx),
        Node::Fragment => render_children(node.children(), res, ctx),
        Node::Text(text) => {
            let txt = text.text.trim();
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
                    },
                );

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
            _ => {
                let mut status = RenderStatus::NotRendered;
                for ch in node.children() {
                    let context = match status {
                        RenderStatus::NotRendered => ctx.merge(ContextType::NewParagraph),
                        RenderStatus::Rendered => Context {
                            context_type: ContextType::Inline,
                            indent: ctx.indent,
                        },
                        RenderStatus::RenderedRequiresSpace => Context {
                            context_type: ContextType::RequiresSpace,
                            indent: ctx.indent,
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

fn render_context(ctx: &Context, first_char: char, res: &mut String) {
    match ctx.context_type {
        ContextType::Inline => (),
        ContextType::RequiresSpace => {
            if first_char != '.' && first_char != ',' && first_char != ';' {
                res.push(' ');
            }
        }
        ContextType::NewParagraph => {
            res.push_str("\n\n");
            for _ in 0..ctx.indent {
                res.push(' ');
            }
        }
        ContextType::UnorderedList => {
            res.push('\n');
            for _ in 0..ctx.indent {
                res.push(' ');
            }
            res.push_str("* ");
        }
        ContextType::OrderedList(idx) => {
            res.push('\n');
            for _ in 0..ctx.indent {
                res.push(' ');
            }
            res.push_str(&format!("{} ", idx));
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
<title>Writing My Own Language - Conclusion | Vid Drobnič</title>

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
                <h1>Writing My Own Language - Conclusion</h1>
                
<div class="flex flex-wrap justify-between -mt-6 prose prose-p:my-0 dark:prose-invert">
    <p class="mr-8 font-light">9 Feb 2025</p>
    <p class="font-light">4 minute read</p>
</div>
<div class="py-3 px-2 mt-6 bg-gray-100 rounded-xl shadow-md prose prose-h3:mt-0 prose-h3:mb-1 prose-li:my-0 prose-ul:my-0 prose-ul:list-none prose-ul:ps-0 prose-li:ps-4 prose-a:no-underline prose-a:font-normal dark:bg-slate-700 dark:prose-invert">
    <h3 class="pl-3.5">Contents</h3>
    <nav id="TableOfContents">
  <ul>
    <li><a href="#language-server-implementation">Language Server Implementation</a>
      <ul>
        <li><a href="#document-symbol-tree">Document Symbol Tree</a></li>
        <li><a href="#location-data">Location Data</a></li>
      </ul>
    </li>
    <li><a href="#using-the-language">Using the Language</a></li>
  </ul>
</nav>
</div>
<p>I decided to write <a href="https://github.com/viddrobnic/aoc-lang">my own programming language</a>
to solve 2024 <a href="https://adventofcode.com/">Advent of Code</a>.
This includes an interpreter, syntax highlighting, and a language server. In the
<a href="https://viddrobnic.com/blog/2024/writing-my-language/">first part</a>, we looked at how the
interpreter works. In the <a href="https://viddrobnic.com/blog/2024/writing-my-language-2/">second part</a>,
we looked at how syntax highlighting is implemented.</p>
<p>In this part we&rsquo;ll take a look at the basic idea behind the LSP implementation and how the language performed
for solving 2024 Advent of Code.</p>
<h2 id="language-server-implementation">
    <a href="#language-server-implementation" class="no-underline peer" style="font-weight: inherit;">
        Language Server Implementation
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h2>
<p>My language server implements the following features:</p>
<ul>
<li><a href="https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_definition">Go to Definition</a></li>
<li><a href="https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_references">Find References</a></li>
<li><a href="https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentHighlight">Document Highlight</a></li>
<li><a href="https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover">Hover</a></li>
<li><a href="https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentSymbol">List Document Symbols</a></li>
<li><a href="https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentSymbol">Completion Recommendations</a></li>
</ul>
<p>When I started this project, I thought I would be able to reuse much of the compiler logic to analyze the AST for the
above queries. However, it turned out that the compiler logic had nothing to do with the AST analysis required for these queries.</p>
<p>I came up with two data structures to resolve these queries: <sup id="fnref:1"><a href="#fn:1" class="footnote-ref" role="doc-noteref">1</a></sup></p>
<ul>
<li><strong>Document Symbol Tree</strong> - A tree of symbols used to provide completion recommendations.</li>
<li><strong>Location Data</strong> - An array of data connected to document locations, used for all other queries.</li>
</ul>
<h3 id="document-symbol-tree">
    <a href="#document-symbol-tree" class="no-underline peer" style="font-weight: inherit;">
        Document Symbol Tree
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h3>
<p>This is a simple data structure - a tree of document symbols. Each document has an array of symbols.
A symbol contains location data, a name, a kind (function/variable), and an array of children.
The children are document symbols within the scope of the parent symbol.</p>
<p>Let&rsquo;s look at an example:</p>
<div class="highlight"><pre tabindex="0" class="chroma"><code class="language-text" data-lang="text"><span class="line"><span class="cl">foo = 420
</span></span><span class="line"><span class="cl">
</span></span><span class="line"><span class="cl">bar = fn() {
</span></span><span class="line"><span class="cl">    baz = 42
</span></span><span class="line"><span class="cl">}
</span></span></code></pre></div><p>In this example, document has two symbols:</p>
<ul>
<li><code>foo</code></li>
<li><code>bar</code></li>
</ul>
<p>The symbol <code>foo</code> has no children, whereas <code>bar</code> has <code>baz</code> as a child.</p>
<h4 id="getting-completion-recommendations">
    <a href="#getting-completion-recommendations" class="no-underline peer" style="font-weight: inherit;">
        Getting Completion Recommendations
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h4>
<p>To get completion recommendations, the text editor sends the language server the cursor location.
With the document symbol tree constructed, we can easily retrieve all symbols that are reachable from the cursor&rsquo;s location.
We then rely on the text editor to filter and sort the recommendations <sup id="fnref:2"><a href="#fn:2" class="footnote-ref" role="doc-noteref">2</a></sup>.</p>
<p>Constructing an array of available document symbols is easy. We just iterate through all symbols and, depending on their location, either:</p>
<ul>
<li>Include the symbol (if it appears before the cursor).</li>
<li>Retrieve the symbol&rsquo;s children (if the cursor is inside a function&rsquo;s scope).</li>
<li>Exclude the symbol (if it appears after the cursor).</li>
</ul>
<h3 id="location-data">
    <a href="#location-data" class="no-underline peer" style="font-weight: inherit;">
        Location Data
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h3>
<p>This is an even simpler data structure - just an array of data connected to document locations, sorted by location.
Because it&rsquo;s sorted, we can query it using <a href="https://en.wikipedia.org/wiki/Binary_search">binary search</a>.</p>
<p>As an example, let&rsquo;s take a look how &ldquo;go to definition&rdquo; is implemented. All other queries follow a similar pattern.</p>
<p>During document analysis, we check whether an identifier has already been defined. If it has, we add its definition&rsquo;s
location to the location data array. If not, we mark it as defined and store the definition’s location.</p>
<p>The result is an array of all symbols and their definition locations, sorted by their position in the document.
To find where a symbol is defined, we simply locate the identifier at the cursor position using binary search and
return its definition&rsquo;s location.</p>
<h2 id="using-the-language">
    <a href="#using-the-language" class="no-underline peer" style="font-weight: inherit;">
        Using the Language
    </a>
    <span class="invisible peer-hover:visible text-blue-500">§</span>
</h2>
<p>I tested the language&rsquo;s usability by solving <a href="https://adventofcode.com/2024">Advent of Code 2024</a> with it. I knew
I wouldn&rsquo;t have time to complete the entire Advent of Code, especially since I was also busy with a new side project.
That&rsquo;s why I decided to solve only the first 10 days. If you’re interested, the solutions are available on
<a href="https://github.com/viddrobnic/adventofcode/tree/master/2024">my GitHub</a>.</p>
<p>After solving the first 10 days with the language, I would say it definitely passed the usability test.
It isn&rsquo;t anywhere near a production-ready language, but it was a very fun side project where I learned a lot.
It&rsquo;s also a side project I can comfortably mark as finished and move on to the next one :)</p>
<div class="footnotes" role="doc-endnotes">
<hr>
<ol>
<li id="fn:1">
<p>I know that this is a solved problem, but I wanted to reinvent the wheel, because sometimes it&rsquo;s fun to do so :)&#160;<a href="#fnref:1" class="footnote-backref" role="doc-backlink">&#x21a9;&#xfe0e;</a></p>
</li>
<li id="fn:2">
<p>This is the &ldquo;lazy&rdquo; way of implementing recommendations. The proper approach would be to handle filtering in
the language server itself. However, that would also require the language server to instruct the editor on exactly
what text to insert, where to insert it, and which existing text to replace.
Since this is just a hobby project, I went with the simpler approach, where the text editor does all the work.&#160;<a href="#fnref:2" class="footnote-backref" role="doc-backlink">&#x21a9;&#xfe0e;</a></p>
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

        render(html).unwrap();
    }
}
