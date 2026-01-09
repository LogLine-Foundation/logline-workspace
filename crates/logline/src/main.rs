use std::env;

use tdln_compiler::{compile, CompileCtx};

fn print_usage() {
    eprintln!("logline CLI\n\nUsage:\n  logline compile <text>   Compile intent → AST/Canon/Proof CIDs\n  logline info             Show stack info\n  logline version          Show version");
}

fn cmd_info() {
    println!("LogLine full bundle: TDLN + JSON✯Atomic + LogLine core + LLLV");
}

fn cmd_version() {
    println!("logline {}", env!("CARGO_PKG_VERSION"));
}

fn cmd_compile(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = CompileCtx {
        rule_set: "v1".into(),
    };
    let compiled = compile(text, &ctx)?;
    let canon_str = String::from_utf8(compiled.canon_json.clone())?;
    println!("kind       : {}", compiled.ast.kind);
    println!("cid        : {}", hex::encode(compiled.cid));
    println!("ast_cid    : {}", hex::encode(compiled.proof.ast_cid));
    println!("canon_cid  : {}", hex::encode(compiled.proof.canon_cid));
    println!("canonical  : {}", canon_str);
    Ok(())
}

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }
    match args.remove(0).as_str() {
        "compile" => {
            if let Some(text) = args.first() {
                if let Err(e) = cmd_compile(text) {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            } else {
                print_usage();
                std::process::exit(1);
            }
        }
        "info" => cmd_info(),
        "version" => cmd_version(),
        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}
