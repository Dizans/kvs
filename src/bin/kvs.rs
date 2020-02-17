use clap::{crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg, SubCommand, AppSettings};
use kvs::KvStore;

fn main() {
    let mut kvs = KvStore::open("db").unwrap();

    let matches = App::new(crate_name!()) //  env!("CARGO_PKG_NAME")
        .setting(AppSettings::ArgRequiredElseHelp)
        .bin_name("kvs")
        .version(crate_version!()) // env!("CARGO_PKG_VERSION")
        .author(crate_authors!()) // env!("CARGO_PKG_AUTHORS")
        .about(crate_description!()) // env!("CARGO_PKG_DESCRIPTION")
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("KEY").index(1).required(true))
                .arg(Arg::with_name("VALUE").index(2).required(true)),
        )
        .subcommand(SubCommand::with_name("get").arg(Arg::with_name("KEY").required(true)))
        .subcommand(SubCommand::with_name("rm").arg(Arg::with_name("KEY").required(true)))
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("set") {
        let key = matches.value_of("KEY").unwrap();
        let value = matches.value_of("VALUE").unwrap();
        kvs.set_with_log(key.to_owned(), value.to_owned()).unwrap();
        return;
    }

    if let Some(ref matches) = matches.subcommand_matches("get") {
        let key = matches.value_of("KEY").unwrap();
        match kvs.get(key.to_owned()){
            Ok(Some(v)) => println!("{}", v),
            Ok(None) => println!("{} does not exists", key),
            Err(_) => println!("an error occurred"),
        };
        return;
    }

    if let Some(ref matches) = matches.subcommand_matches("rm") {
        let key = matches.value_of("KEY").unwrap();
        kvs.remove_with_log(key.to_owned()).unwrap();
        return;
    }

    // match matches.subcommand_name() {
    //     Some("set") => {
    //         KvStore.get(key: String)
    //     }
    //     Some("get") => {}
    //     Some("rm") => {}
    //     None => {}
    //     _ => {}
    // }
    // println!("Hello, world!");
    
}
