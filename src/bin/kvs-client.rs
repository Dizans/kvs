use clap::{crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg, SubCommand, AppSettings};
use kvs::KvStore;
use std::env::current_dir;
use std::net::SocketAddr;

macro_rules! addr_arg {
    () => {
        Arg::with_name("addr")
        .takes_value(true)
        .multiple(false)
        .help("--addr IP:PORT")
        .long("addr")
    }
}

fn main() {
    let current_dir = current_dir().unwrap();
    let mut kvs = KvStore::open(current_dir).unwrap();

    let matches = App::new(crate_name!()) //  env!("CARGO_PKG_NAME")
        .setting(AppSettings::ArgRequiredElseHelp)
        .bin_name("kvs-client")
        .version(crate_version!()) // env!("CARGO_PKG_VERSION")
        .author(crate_authors!()) // env!("CARGO_PKG_AUTHORS")
        .about(crate_description!()) // env!("CARGO_PKG_DESCRIPTION")
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("KEY").index(1).required(true))
                .arg(Arg::with_name("VALUE").index(2).required(true))
                .arg(
                    addr_arg!()
                ),
        )
        .subcommand(SubCommand::with_name("get")
                    .arg(Arg::with_name("KEY").required(true))
                    .arg(addr_arg!()))
        .subcommand(SubCommand::with_name("rm")
                    .arg(Arg::with_name("KEY").required(true))
                    .arg(addr_arg!()))
        .get_matches();
    
    let mut addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    
    if let Some(ref matches) = matches.subcommand_matches("set") {
        let key = matches.value_of("KEY").unwrap();
        let value = matches.value_of("VALUE").unwrap();

        match_addr(&matches, &mut addr);
        kvs.set(key.to_owned(), value.to_owned()).unwrap();
        println!("{}", addr);
        return;
    }

    if let Some(ref matches) = matches.subcommand_matches("get") {
        let key = matches.value_of("KEY").unwrap();
        match_addr(&matches, &mut addr);
        match kvs.get(key.to_owned()){
            Ok(Some(v)) => println!("{}", v),
            Ok(None) => println!("Key not found"),
            Err(_) => println!("an error occurred"),
        };
        return;
    }

    if let Some(ref matches) = matches.subcommand_matches("rm") {
        let key = matches.value_of("KEY").unwrap();
        match_addr(&matches, &mut addr);
        match kvs.remove(key.to_owned()){
            Ok(_) => {},
            Err(_) => {
                println!("Key not found");
                std::process::exit(2);
            }
        }
        return;
    }

}

fn set(key: String, value: String){

}

fn get(key: String) -> Option<String>{
    Some(format!(""))
}

fn rm(key: String){

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
