extern crate voile;

use voile::check::check_main;
use voile::syntax::abs::trans_decls;

mod args;
mod repl;
mod util;

fn main() {
    let args = args::pre();

    let checked = args
        .file
        .clone()
        .and_then(|s| util::parse_file(s.as_str()))
        .map(|ast| {
            if !args.quiet {
                println!("Parse successful.");
            }

            if !args.parse_only {
                // Translate to abstract syntax
                let abs = trans_decls(ast)
                    .map_err(|err| eprintln!("{}", err))
                    .unwrap_or_else(|()| {
                        eprintln!("Translate failed.");
                        std::process::exit(1)
                    });

                // Type Check
                let checked = check_main(abs)
                    .map_err(|err| eprintln!("{}", err))
                    .unwrap_or_else(|()| {
                        eprintln!("Type-Check failed.");

                        std::process::exit(1);
                    });

                if !args.quiet {
                    println!("Type-Check successful.");
                }

                checked
            } else {
                Default::default()
            }
        })
        .unwrap_or_else(Default::default);

    // REPL
    if args.interactive_plain {
        repl::repl_plain(checked)
    } else if args.interactive {
        repl::repl(checked)
    }
}
