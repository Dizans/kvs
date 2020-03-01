# kvs
rust implementation of  a simple key-value database

# usage
Run a kvs server: 
  `cargo run --bin kvs-server -- --addr {IP:PORT} --engine {ENGINE}`
If --engine is specified, then ENGINE-NAME must be either "kvs" or "sled".By default, it's "kvs".

For example, you could start a server in terminal: 
  `cargo run --bin kvs-server -- --addr "127.0.0.1:8899" --engine kvs`.
  
To send a send a message to kvs server

  set: `cargo run --bin kvs-client set "answer" "42" --addr "127.0.0.1:8899"` 
  
  get: `cargo run --bin kvs-client get "answer" --addr "127.0.0.1:8899"`
  
  remove: `cargo run --bin kvs-client rm "answer" --addr "127.0.0.1:8899"`
