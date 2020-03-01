use clap::{crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg, SubCommand, AppSettings};
use std::net::{SocketAddr, TcpStream};
use std::io::prelude::*;
use std::str;
use kvs::*;

fn main() {
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
                    addr_arg()
                ),
        )
        .subcommand(SubCommand::with_name("get")
                    .arg(Arg::with_name("KEY").required(true))
                    .arg(addr_arg()))
        .subcommand(SubCommand::with_name("rm")
                    .arg(Arg::with_name("KEY").required(true))
                    .arg(addr_arg()))
        .get_matches();
    
    let mut addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    
    if let Some(ref matches) = matches.subcommand_matches("set") {
        let key = matches.value_of("KEY").unwrap();
        let value = matches.value_of("VALUE").unwrap();

        match_addr(&matches, &mut addr);
        let command = Command::Set(key.to_owned(), value.to_owned());
        let msg = serde_json::to_string(&command).unwrap();
        call_server(&addr, &msg);
        return;
    }

    if let Some(ref matches) = matches.subcommand_matches("get") {
        let key = matches.value_of("KEY").unwrap();
        match_addr(&matches, &mut addr);
        let command = Command::Get(key.to_owned());
        let msg = serde_json::to_string(&command).unwrap();
        let res = call_server(&addr, &msg);
        match res{
            Response::Error(ServerError::NotFound) => println!("Key not found"),
            Response::Value(s) => println!("{}", s),
            _ => {},
        }
        return;
    }

    if let Some(ref matches) = matches.subcommand_matches("rm") {
        let key = matches.value_of("KEY").unwrap();
        match_addr(&matches, &mut addr);
        let command = Command::Rm(key.to_owned());
        let msg = serde_json::to_string(&command).unwrap();
        let res = call_server(&addr, &msg);
        match res{
            Response::Error(ServerError::NotFound) => {
                eprintln!("Key not found");
                std::process::exit(1);
            },
            _ => {},
        }
        return;
    }
}

fn call_server(addr: &SocketAddr, msg: &str) -> Response{
    let mut stream = TcpStream::connect(addr)
                    .unwrap_or_else(|e| panic!("connect to server failed: {}", e));
    stream.write(msg.as_bytes()).unwrap();

    let mut buf = [0; 512];
    
    let len = stream.read(&mut buf).unwrap();
    let response: Response = serde_json::from_slice(&buf[0..len]).unwrap(); 
    response
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

fn addr_arg() -> clap::Arg<'static,'static>{
    Arg::with_name("addr")
    .takes_value(true)
    .multiple(false)
    .help("--addr IP:PORT")
    .long("addr")
}

