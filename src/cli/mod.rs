extern crate voile;

use voile::check::check_decls;
use voile::check::monad::TCS;
use voile::syntax::abs::trans_decls_contextual;

use crate::repl::code_to_abs;

mod args;
mod repl;
mod util;

fn main() {
    let args = args::pre();

    let mut checked = args
        .file
        .clone()
        .and_then(|s| util::parse_file(s.as_str()))
        .map(|decls| {
            if !args.quiet {
                println!("Parse successful.");
            }

            if !args.parse_only {
                // Translate to abstract syntax
                let abs_decls = trans_decls_contextual(Default::default(), decls)
                    .map_err(|err| eprintln!("{}", err))
                    .unwrap_or_else(|()| {
                        eprintln!("Scope-Check failed.");
                        std::process::exit(1)
                    });

                // Type Check
                let mut tcs = TCS::default();
                tcs.expand_with_fresh_meta(abs_decls.meta_count);
                let checked = check_decls(tcs, abs_decls.decls.clone())
                    .map_err(|err| eprintln!("{}", err))
                    .unwrap_or_else(|()| {
                        eprintln!("Type-Check failed.");
                        std::process::exit(1)
                    });

                if !args.quiet {
                    for i in 0..checked.glob_len() {
                        println!("sign: {}", checked.gamma[i].ast);
                        println!("body: {}", checked.env[i].ast);
                    }

                    // Meme: https://github.com/owo-lang/voile-rs/issues/56
                    println!("Checkmate, dram!");
                }

                (checked, abs_decls)
            } else {
                Default::default()
            }
        })
        .unwrap_or_else(Default::default);

    if let Some(abs) = args
        .evaluate
        .and_then(|code| code_to_abs(&mut checked, &code))
    {
        let (tcs, trans_st) = checked;
        let (core, tcs) = tcs.evaluate(abs);
        println!("{}", core.ast);
        checked = (tcs, trans_st);
    }

    // REPL
    if args.interactive_plain {
        repl::repl_plain(checked)
    } else if args.interactive {
        repl::repl(checked)
    }
}
