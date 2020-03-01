use std::net::{
    SocketAddr,
    TcpListener,
    TcpStream
};
use std::io::{
    Write,
    prelude::*,
};
use std::path::PathBuf;

use log::{info, error};
use clap::{crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg, AppSettings};
use sled;

use kvs::{Engine,Command,Response, ServerError,KvsError,KvsEngine, KvStore, SledStore};

fn main() { 
    kvs::log_init();
    let matches = get_cli_mathces();

    let mut addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    match_addr(&matches, &mut addr);

    let engine_exists = engine_file_exists();
    let engine_specified = match_engine(&matches);

    let engine;
    match (engine_exists, engine_specified){
        (None, None) => engine = Engine::Kvs,
        (None, Some(e)) => engine = e,
        (Some(e), None) => engine = e,
        (Some(e1), Some(e2)) => {
            if e1 != e2{
                error!("Engine {:?} file already exists, ", e1);
                std::process::exit(2);
            }
            engine = e1;
        }
    }

    let server = Server::new(addr, engine);
    server.run();
}

fn get_cli_mathces() -> clap::ArgMatches<'static>{
    App::new(crate_name!()) //  env!("CARGO_PKG_NAME")
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
        .get_matches()
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

fn match_engine(matches: &clap::ArgMatches) -> Option<Engine> {
     match matches.value_of("engine"){
        None => None,
        Some("kvs") =>  Some(Engine::Kvs),
        Some("sled") => Some(Engine::Sled),
        _ => {
            eprintln!("Invalid engine value, see help.");
            std::process::exit(1);
        }
    }
}

fn engine_file_exists() -> Option<Engine>{
    let path = PathBuf::from("./kvstore");
    if path.exists() && path.is_dir(){
        return Some(Engine::Kvs);
    }

    let path = PathBuf::from("./sled");
    if path.exists() && path.is_dir(){
        return Some(Engine::Sled);
    }
    
    return None;
}

struct Server{
    addr: SocketAddr,
    engine: Engine,
}

impl Server{
    pub fn new(addr: SocketAddr, engine: Engine) -> Self{
        Server{
            addr,
            engine,
        }
    }

    pub fn run(&self) {
        info!("starting server, version: {}", crate_version!());
        info!("server started at {}, engine: {:?}", self.addr, self.engine);
        
       match self.engine{
            Engine::Kvs => {
                let mut engine = KvStore::open("kvstore").unwrap();
                self.handle_with_engine(&mut engine);
            },

            Engine::Sled => {
                let mut engine = SledStore::new(sled::open("sled_store").unwrap());
                self.handle_with_engine(&mut engine);
            }
        };
    }

    fn handle_with_engine<E: KvsEngine>(&self, engine: &mut E){
        let listener = TcpListener::bind(self.addr)
                        .unwrap_or_else(|e| panic!("bind server failed: {}", e));

        for stream in listener.incoming(){
            let mut stream = stream.unwrap();
            let mut buffer = [0; 512];

            let len = stream.read(&mut buffer).unwrap();
            let command: Result<Command, _> = serde_json::from_slice(&buffer[0..len]);

            let response;
            match command{
                Ok(op) => {
                    response = self.do_command(engine, op);
                },
                Err(_) => {
                    response = Response::Error(ServerError::InvalidCommand);
                }
            }

            let res = serde_json::to_string(&response).unwrap();
            stream.write(&res.into_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }


    fn do_command<E: KvsEngine>(&self, engine: &mut E, op: Command) -> Response{
       match op{
           Command::Set(k, v) => {
               match engine.set(k, v){
                   Err(_) => {
                       Response::Error(ServerError::OtherError)
                   },
                   Ok(_) => {
                       Response::Null
                   }
               }
           },

           Command::Get(k) => {
               match engine.get(k){
                    Ok(v) => {
                        match v{
                            Some(s) => Response::Value(s),
                            None => Response::Error(ServerError::NotFound)
                        }
                    }
                    Err(_) => Response::Error(ServerError::OtherError),
               }
           },

           Command::Rm(k) => {
                match engine.remove(k){
                    Ok(_) => Response::Null,
                    Err(e) => {
                        match e{
                            KvsError::NotFound(_) => Response::Error(ServerError::NotFound),
                            _ => Response::Error(ServerError::OtherError),
                        }
                    }
                }
           }
       }
   }
}


