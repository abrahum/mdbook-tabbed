use clap::{App, Arg, SubCommand};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
mod tabbed;

fn make_app() -> App<'static, 'static> {
    App::new("tabbed")
        .about("A mdbook preprocessor which processes tabs")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();
    let tabbed = tabbed::Tabbed {};
    if let Some(_sub_args) = matches.subcommand_matches("supports") {
        println!("yes");
    } else if let Err(e) = preprocessing(&tabbed) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn preprocessing(pre: &dyn Preprocessor) -> Result<(), mdbook::errors::Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(std::io::stdin())?;
    pre.run(&ctx, book.clone())?;
    serde_json::to_writer(std::io::stdout(), &book)?;
    Ok(())
}
