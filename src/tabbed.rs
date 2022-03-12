use mdbook::{
    book::{Book, Chapter},
    errors::Error as MdError,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use pulldown_cmark::{CowStr, Event, Parser, Tag};

const SCRIPT: &str = r#"
<style>
    .tabbed-labels>input {
        display: none;
    }
</style>
<script>
    window.onload = () => {
        const tabs = document.querySelectorAll(".tabbed-labels>input");
        for (const tab of tabs) {
            tab.addEventListener("click", () => {
                if (tab.checked) {
                    return
                }
                const inputs = tab.closest(".tabbed-labels").querySelectorAll("input");
                for (const input of inputs) {
                    if (input.checked) {
                        console.log(input.id);
                        document.querySelector(`#${input.id}_div`).style.display = "none";
                        input.checked = false
                    }
                }
                document.querySelector(`#${tab.id}_div`).style.display = "block";
                tab.checked = true
            })
        }
    }
</script>"#;

pub(crate) struct Tabbed {
    script: Vec<Event<'static>>,
}

#[derive(Debug)]
struct Tab {
    title: String,
    start: usize,
    end: usize,
}

impl Preprocessor for Tabbed {
    fn name(&self) -> &str {
        "tabbed"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, MdError> {
        book.for_each_mut(|book_item| {
            if let BookItem::Chapter(chapter) = book_item {
                if let Err(e) = self.tabbed(chapter) {
                    eprintln!("tabbed error: {:?}", e);
                }
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, name: &str) -> bool {
        name == "html"
    }
}

impl Tabbed {
    fn tabbed(&self, chapter: &mut Chapter) -> Result<(), MdError> {
        let events: Vec<Event> = Parser::new_ext(
            &chapter.content,
            pulldown_cmark::Options::from_bits_truncate(0b111111),
        )
        .collect();
        let mtabs = get_multi_tabs(&events);
        eprintln!("{:?}", mtabs);
        // eprintln!("{:#?}", events);
        let mut nes = new_events(events, &mtabs);
        nes.extend(self.script.clone());
        let mut out_put = String::new();
        pulldown_cmark_to_cmark::cmark(nes.into_iter(), &mut out_put)?;
        eprintln!("{}", out_put);
        chapter.content = out_put;
        Ok(())
    }

    pub fn new() -> Self {
        Self {
            script: Parser::new(SCRIPT).collect(),
        }
    }
}

fn get_multi_tabs(events: &Vec<Event>) -> Vec<Vec<Tab>> {
    let mut multi_tabs = vec![];
    let len = events.len();
    let mut i = 0;
    while let Some(tabs) = get_tabs(events, &mut i, len) {
        multi_tabs.push(tabs);
    }
    multi_tabs
}

fn get_tabs(events: &Vec<Event>, s: &mut usize, end: usize) -> Option<Vec<Tab>> {
    while *s < end {
        match events[*s] {
            Event::Start(Tag::Paragraph) => {
                let mut tabs = vec![];
                while let Some(tab) = Tab::new(events, s) {
                    tabs.push(tab);
                    if *s + 1 >= end {
                        break;
                    } else if let Event::Start(Tag::Paragraph) = events[*s + 1] {
                        *s += 1;
                    } else {
                        break;
                    }
                }
                if tabs.len() > 1 {
                    return Some(tabs);
                }
            }
            _ => {}
        }
        *s += 1;
    }
    None
}

impl Tab {
    pub fn new(events: &Vec<Event>, s: &mut usize) -> Option<Self> {
        *s += 1;
        if let Event::Text(CowStr::Borrowed(title)) = events[*s] {
            let v: Vec<_> = title.split("").collect();
            if v.len() > 4 {
                if v[0..5].join("") == "=== " {
                    let title = v[5..v.len() - 1].join("");
                    *s += 1;
                    if let Event::End(Tag::Paragraph) = events[*s] {
                        *s += 1;
                        let (start, end) = skip_tag(events, s);
                        return Some(Tab { title, start, end });
                    }
                }
            }
        }
        None
    }

    pub fn input(&self, i: usize, j: usize, default: bool) -> Event {
        Event::Html(
            format!(
                r#"<input{} id="__tabbed_{}_{}">"#,
                if default { r#" checked="true""# } else { "" },
                i,
                j
            )
            .into(),
        )
    }

    pub fn label(&self, i: usize, j: usize) -> Event {
        Event::Html(
            format!(
                r#"<label for="__tabbed_{}_{}">{}</label>"#,
                i, j, self.title
            )
            .into(),
        )
    }

    pub fn content(&self, i: usize, j: usize, default: bool) -> Event {
        Event::Html(
            format!(
                r#"<div id="__tabbed_{}_{}_div"{}>"#,
                i,
                j,
                if !default {
                    r#" style="display: none;""#
                } else {
                    ""
                },
            )
            .into(),
        )
    }
}

fn skip_tag(events: &Vec<Event>, s: &mut usize) -> (usize, usize) {
    let i = *s;
    let mut tag = None;
    loop {
        match &events[*s] {
            Event::Start(t) => {
                if tag.is_none() {
                    tag = Some(t);
                }
            }
            Event::End(t) => {
                if let Some(ot) = tag {
                    if ot == t {
                        break;
                    }
                }
            }
            // Event::Html(_) => {
            //     *s += 1;
            //     while let Event::Html(_) = events[*s] {
            //         *s += 1;
            //     }
            //     *s -= 1;
            //     break;
            // }
            _ => {}
        }
        *s += 1;
    }
    (i, *s)
}

fn new_events<'a>(events: Vec<Event<'a>>, mtabs: &'a Vec<Vec<Tab>>) -> Vec<Event<'a>> {
    let mut index = 0usize;
    let mut new_events = vec![];
    for (i, tabs) in mtabs.iter().enumerate() {
        new_events.extend(events[index..tabs[0].start - 2].to_vec());
        new_events.push(Event::Html(r#"<div class="tabbed-set">"#.into()));

        new_events.push(Event::Html(r#"<div class="tabbed-labels">"#.into()));
        let inputs = tabs
            .iter()
            .enumerate()
            .map(|(j, tab)| tab.input(i, j, j == 0))
            .collect::<Vec<_>>();
        new_events.extend(inputs);
        let labels = tabs
            .iter()
            .enumerate()
            .map(|(j, tab)| tab.label(i, j))
            .collect::<Vec<_>>();
        new_events.extend(labels);
        new_events.push(Event::Html(r#"</div>"#.into()));
        let contents = tabs
            .iter()
            .enumerate()
            .map(|(j, tab)| tab.content(i, j, j == 0))
            .collect::<Vec<_>>();
        new_events.push(Event::Html(r#"<div class="tabbed-contents">"#.into()));
        for i in 0..tabs.len() {
            new_events.push(contents[i].clone());
            new_events.push(Event::HardBreak);
            new_events.extend(events[tabs[i].start..tabs[i].end + 1].to_vec());
            new_events.push(Event::Html(r#"</div>"#.into()));
        }
        new_events.push(Event::Html(r#"</div>"#.into()));
        new_events.push(Event::Html(r#"</div>"#.into()));
        new_events.push(Event::HardBreak);
        index = tabs[tabs.len() - 1].end + 1;
    }
    new_events.extend(events[index..].to_vec());
    new_events
}
