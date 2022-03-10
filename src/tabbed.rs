use mdbook::{
    book::{Book, Chapter},
    errors::Error as MdError,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use pulldown_cmark::{Event, Parser};

pub(crate) struct Tabbed {}

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
        println!("chapter: {:?}", chapter);
        let events: Vec<Event> = Parser::new(&chapter.content).collect();
        println!("{:?}", events);
        Ok(())
    }
}
