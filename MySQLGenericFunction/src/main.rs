use log::{debug, info};
use std::{env};
use std::collections::HashMap;
use std::time::SystemTime;
use clap::Parser;
use mysql::prelude::*;
use mysql::*;

//Usage env parameters --operation {CRUD operation} --id {ID} --rev {REV} --table {TABLE} --firstname {FIRSTNAME} --lastname {LASTNAME}  --firstname-op {FIRSTNAME-OP} --lastname-op {LASTNAME-OP}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Operation name
    #[clap(short, long, default_value = "Read")]
    operation: String,

    // Table name
    #[clap(short, long, default_value = "Customers" )]
    table: String,

    // id of the entry to create/read/update/delete
    #[clap(long, default_value = "" )]
    id:String,

    // _ref of the entry to update usefull in CouchDB
    #[clap(long, default_value = "")]
    rev:String,

    // Firstname
    #[clap(long, default_value = "" )]
    firstname: String,

    // Lastname
    #[clap(long, default_value = "" )]
    lastname: String,

    // Firstname to Update
    #[clap(long, default_value = "" )]
    firstname_opt: String,

    // Lastname to Update
    #[clap(long, default_value = "" )]
    lastname_opt: String,
}

#[derive(Debug, PartialEq, Clone)]
enum Op {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Clone)]
struct Request {
    op: Op,
    table : String,
    param: HashMap<String, String>,
    param_to_up: Option<HashMap<String, String>>,
}

// Convert a Request to a MySQL query
fn from_request_to_query(request: Request) -> String {
    debug!("Invoker | type of operation requested: {:?}", request.op);
    match request.op {
        Op::Create => {
            let mut col:String = String::new();
            let mut val :String = String::new();
            let mut first = true;

            //  Split name, val
            for p in request.param {
                if first{
                    col = format!("{}", p.0);
                    val = format!("'{}'", p.1);
                    first = false;
                }else{
                    col = format!("{},{}", col, p.0);
                    val = format!("{},'{}'", val, p.1);
                }
            }
            format!("INSERT INTO {} ({}) VALUES ({});", request.table, col, val)
        },
        Op::Read => {
            let mut to_find:String = String::new();
            let mut first = true;
            for p in request.param {
                if first {
                    to_find = format!("{}='{}' ", p.0, p.1);
                    first = false;
                }else{
                    to_find = format!("{}AND {}='{}'", to_find, p.0, p.1);
                }
            }
            format!("SELECT * FROM {} WHERE {};", request.table, to_find )
        },
        Op::Update => {
            let mut old_entry:String = String::new();
            let mut new_entry:String = String::new();
            let mut first = true;
            let mut first_new = true;

            //	Data to modify
            for p in request.param {
                if first {
                    old_entry = format!("{}='{}'", p.0, p.1);
                    first = false;
                }else {
                    old_entry = format!("{} AND {}='{}'", old_entry, p.0, p.1);
                }
            }
            //  New Data
            for p in request.param_to_up.unwrap() {
                if first_new {
                    new_entry = format!("{}='{}'", p.0, p.1);
                    first_new = false;
                }else {
                    new_entry = format!("{},{}='{}'", new_entry, p.0, p.1);
                }
            }

            format!("UPDATE {} SET {} WHERE {};",request.table, new_entry, old_entry)
        },
        Op::Delete => {
            let mut to_delete:String = String::new();
            let mut first = true;
            for p in request.param {
                if first {
                    to_delete = format!("{}='{}'", p.0, p.1);
                    first = false;
                }
                else {
                    to_delete = format!("{} AND {}='{}'", to_delete, p.0, p.1);
                }
            }
            format!("DELETE from {} where {};", request.table, to_delete)
        },
    }
}

// Execute the operation on DB
fn execute_operation(mut conn: mysql::PooledConn, req: Request) -> String{
    //  Query generation
    let query = from_request_to_query(req.clone());
    debug!("Query to execute: {}", query);

    //  Operation execution
    let res = match req.op{
        Op::Read =>{
            // Send back a Vec<Row> to keep the invoker independent from the data type
            let start_time = SystemTime::now();
            let query_result :Vec<Row> = conn.query(query).unwrap();
            let db_duration = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_duration.as_micros());

            let mut query_string :String = String::new();
            for el in query_result{
                query_string = format!("{} {:?}",query_string, el);
            }
            query_string
        },
        Op::Create | Op::Update | Op::Delete => {
            let start_time = SystemTime::now();
            let query_result =
                match conn.query_drop(query){
                    Ok(_) => String::from("Success"),
                    Err(_) =>String::from("Error")
                };
            let duration = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", duration.as_micros());

            query_result
        },
    };
    res
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    // DB parameters should be provided through environment or as at command invocation (the invoker provides the command args at launch)?
    let username = env::var("NAME").unwrap_or("root".to_string());
    let password = env::var("PASSWORD").unwrap_or("root".to_string());
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("3306".to_string());
    let db = env::var("DB-NAME").unwrap_or("testDB".to_string());
    let url_db = format!("mysql://{}:{}@{}:{}/{}", username, password, address, port, db);

    let mut customer: HashMap<String, String> = HashMap::new();
    let mut customer_new: HashMap<String, String> = HashMap::new();

    // Connection to DB
    let opts = Opts::from_url(url_db.as_str());
    let pool = Pool::new(opts.unwrap()).unwrap();
    let conn = pool.get_conn().unwrap();
    debug!("Connected to DB: {:?}", conn);

    // added an id column (not auto increment) in the DBs so the client can add it manually
    if args.id != "".to_string() {
        customer.insert("id".to_string(), args.id);
    }
    if args.firstname != "".to_string() {
        customer.insert("FIRSTNAME".to_string(), args.firstname);
    }
    if args.lastname != "".to_string() {
        customer.insert("LASTNAME".to_string(), args.lastname);
    }
    if args.firstname_opt != "".to_string() {
        customer_new.insert("FIRSTNAME".to_string(), args.firstname_opt);
    }
    if args.lastname_opt != "".to_string() {
        customer_new.insert("LASTNAME".to_string(), args.lastname_opt);
    }

    //  Request building
    debug!("Operation selected: {}", args.operation);
    let req =
        match args.operation.as_str() {
            "Create" =>
                Request {
                    op: Op::Create,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Update
            "Update" =>
                Request {
                    op: Op::Update,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Delete
            "Delete" =>
                Request {
                    op: Op::Delete,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Read
            "Read" | _ =>
                Request {
                    op: Op::Read,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
        };
    debug!("Request: {:?}", req);

    //  Operation execution
    let res  = execute_operation(conn, req);

    //  Send the result to the invoker
    println!("{}", res);
}
