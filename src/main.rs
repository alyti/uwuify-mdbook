use crate::uwuify_lib::UwUifier;
use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use std::io;
use std::process;

const NAME: &str = "uwuify";

pub fn make_app() -> App<'static, 'static> {
    App::new("uwuify-preprocessor")
        .about("A mdbook preprocessor which uwuifies your books uwu")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    // Users will want to construct their own preprocessor here
    let preprocessor = UwUifier::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

mod uwuify_lib {
    use mdbook::{book::Chapter, BookItem};
    use pulldown_cmark::{Event, Parser};
    use pulldown_cmark_to_cmark::cmark;

    use super::*;

    /// A no-op preprocessor.
    pub struct UwUifier;

    impl UwUifier {
        pub fn new() -> UwUifier {
            UwUifier
        }
    }

    impl Preprocessor for UwUifier {
        fn name(&self) -> &str {
            NAME
        }

        fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
            process(&mut book.sections)?;
            Ok(book)
        }

        fn supports_renderer(&self, renderer: &str) -> bool {
            renderer != "not-supported"
        }
    }

    fn process<'a, I>(items: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = &'a mut BookItem> + 'a,
    {
        for item in items {
            if let BookItem::Chapter(ref mut chapter) = *item {
                eprintln!("{}: processing chapter '{}'", NAME, chapter.name);

                let md = uwuify(chapter)?;
                chapter.content = md;

                if !chapter.sub_items.is_empty() {
                    process(&mut chapter.sub_items)?;
                }
            }
        }

        Ok(())
    }

    fn uwuify(chapter: &mut Chapter) -> Result<String, Error> {
        let mut buf = String::with_capacity(chapter.content.len());
        let events = Parser::new(&chapter.content).map(|e| -> Event {
            match e {
                Event::Text(s) => {
                    let b = s.as_bytes();
                    let mut temp1 = vec![0u8; uwuifier::round_up16(b.len()) * 16];
                    let mut temp2 = vec![0u8; uwuifier::round_up16(b.len()) * 16];
                    let res = uwuifier::uwuify_sse(b, &mut temp1, &mut temp2);
                    Event::Text(pulldown_cmark::CowStr::from(
                        std::str::from_utf8(res).unwrap().to_string(),
                    ))
                }
                e => e,
            }
        });

        cmark(events, &mut buf, None)
            .map(|_| buf)
            .map_err(|err| Error::new(err))
    }
}
