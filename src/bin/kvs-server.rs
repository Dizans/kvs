use clap::{crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg, AppSettings};
use std::net::SocketAddr;

fn main() {
    let matches = App::new(crate_name!()) //  env!("CARGO_PKG_NAME")
        .setting(AppSettings::ArgRequiredElseHelp)
        .bin_name("kvs-server")
        .version(crate_version!()) // env!("CARGO_PKG_VERSION")
        .author(crate_authors!()) // env!("CARGO_PKG_AUTHORS")
        .about(crate_description!()) // env!("CARGO_PKG_DESCRIPTION")
        .arg(Arg::with_name("addr")
                .takes_value(true)
                .multiple(false)
                .required(true)
                .help("--addr IP:PORT")
                .long("addr")
        )
        .arg(
            Arg::with_name("engine")
            .takes_value(true)
            .multiple(false)
            .help("--engine ENGINE-NAME")
            .help("the ENGINE-NAME is either \"kvs\" or \"sled\"")
            .long("engine")
        )
        .get_matches();
    
    let mut addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    match_addr(&matches, &mut addr);

    let engine_exists = engine_file_exists();

    let mut engine_specified = None;
    match matches.value_of("engine"){
        None => {},
        Some("kvs") => engine_specified = Some(Engine::Kvs),
        Some("sled") => engine_specified = Some(Engine::Sled),
        _ => {
            eprintln!("Invalid engine value, see help.");
            std::process::exit(1);
        }
    }

    let engine;
    match (engine_exists, engine_specified){
        (None, None) => engine = Engine::Kvs,
        (None, Some(e)) => engine = e,
        (Some(e), None) => engine = e,
        (Some(e1), Some(e2)) => {
            if e1 != e2{
                eprintln!("Engine {:?} file already exists, ", e1);
                std::process::exit(2);
            }
            engine = e1;
        }
    }

    println!("engine {:?}", engine);
}

fn match_addr(matches: &clap::ArgMatches, addr:&mut std::net::SocketAddr){
  if matches.is_present("addr"){
      let s = matches.value_of("addr").unwrap();
      *addr = match s.parse(){
          Ok(v) => v,
          Err(_) => {
              eprintln!("Invalid addr");
              std::process::exit(1);
          }
      };
  }
}

fn engine_file_exists() -> Option<Engine>{
    // TODO
    None
}

#[derive(PartialEq,Debug)]
enum Engine{
    Kvs,
    Sled,
}
