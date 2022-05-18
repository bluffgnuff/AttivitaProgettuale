use log::{debug, info};
use reqwest::Client;
use std::{env};
use std::collections::HashMap;
use std::time::SystemTime;
use clap::Parser;

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

    // id of the entry to read/update/delete
    #[clap(long, default_value = "" )]
    id:String,

    // _ref of the entry to update
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

    // Json document
    #[clap(long, default_value = "" )]
    json:String,
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
    table: String,
    param: HashMap<String, String>,
    param_to_up: Option<HashMap<String, String>>,
}

//  Convert a Request to a CouchDB query
fn from_request_to_json(request: Request) -> String {
    debug!("Type of operation requested: {:?}", request.op);
    match request.op {
        Op::Create => {
            format!("{:?}" ,request.param)
        },
        Op::Read => {
            let mut to_find:String = String::new();
            let mut first = true;
            let start_select = "{\"selector\": {";
            let close = "}";
            let eq =  ": {\"$eq\":";

            to_find = format!("{} {}", to_find, start_select);
            if request.clone().param.len() > 1{
                to_find = format!("{}", to_find);
            }
            for p in request.param.clone() {
                if first {
                    to_find = format!("{} \"{}\" {} \"{}\"{}", to_find, p.0, eq, p.1, close);
                    first = false;
                }else{
                    to_find = format!("{}, \"{}\" {} \"{}\"{}", to_find, p.0, eq, p.1, close);
                }
            }
            if request.param.len() > 1{
                to_find = format!("{}", to_find);
            }
            format!("{}{}{}", to_find, close, close)
        },
        Op::Update => {
            let start = "{";
            let close = "}";
            let old_rev = request.param.get("_rev").unwrap();
            let mut res = format!("{} \"_rev\": \"{}\"", start, old_rev);
            for (key, val) in request.param.clone() {
                if key!= "_rev".to_string() && key!= "_id".to_string() {
                    res = format!("{}, \"{}\": \"{}\"", res, key, val);
                }
            }
            format!("{} {}", res, close)
        },
        Op::Delete  => format!("")
    }
}

//  Execute the operation on DB
async fn execute_operation(client: Client, url_base_db: String, username: String, password: String, req: Request, data: String) -> String{
    let res = match req.op{
        Op::Create => {
            let query_result = client.post(url_base_db).basic_auth(username, Some(password)).body(data).header("Content-Type", "application/json").send().await.unwrap().text().await.unwrap();
            query_result
        },
        Op::Read =>{
            let url= format!("{}/_find",url_base_db);

            let query_result = client.post(url).basic_auth(username, Some(password)).body(data).header("Content-Type", "application/json").send().await.unwrap().text().await.unwrap();
            query_result
        },
        Op::Update => {
            let url= format!("{}/{}",url_base_db, req.param.get("id").unwrap() );

            let query_result = client.put(url).basic_auth(username, Some(password)).body(data).header("Content-Type", "application/json").send().await.unwrap().text().await.unwrap();
            query_result
        },
        Op::Delete => {
            let url= format!("{}/{}",url_base_db, data);

            let query_result = client.delete(url).basic_auth(username, Some(password)).header("Content-Type", "application/json").send().await.unwrap().text().await.unwrap();
            query_result
        },
    };
    res
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    //  DB parameters should be provided through environment or as at command invocation (the invoker provides the command args at launch)?
    let username = env::var("NAME").unwrap_or("root".to_string());
    let password = env::var("PASSWORD").unwrap_or("root".to_string());
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("5984".to_string());
    let db = env::var("DB-NAME").unwrap_or("testdb".to_string());
    let url_base_db = format!("http://{}:{}/{}", address, port, db);
    let mut customer: HashMap<String, String> = HashMap::new();
    let mut customer_new: HashMap<String, String> = HashMap::new();

    //  added an id column (not auto increment) in the DBs so the client can add it manually
    if args.id != "".to_string() {
        customer.insert("id".to_string(), args.id);
    }

    if args.rev != "".to_string() {
        customer.insert("_rev".to_string(),args.rev);
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

    debug!("Operation selected: {}", args.operation);

    //  Request building
    let req =
        match args.operation.as_str() {
            "Create" =>
                Request {
                    op: Op::Create,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            "Update" =>
                Request {
                    op: Op::Update,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            "Delete" =>
                Request {
                    op: Op::Delete,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            "Read" | _ =>
                Request {
                    op: Op::Read,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
        };
    debug!("Request: {:?}", req);

    //  Client HTTP
    let client = Client::builder().build().unwrap();
    debug!("Client created to DB: {:?}", client);

    //  Json Document Generation
    let data = from_request_to_json(req.clone());
    debug!("Json: {:?}", data);

    //  Operation execution
    let start_time = SystemTime::now();
    let res  = execute_operation(client, url_base_db, username, password, req.clone(), data).await;
    let db_duration = SystemTime::now().duration_since(start_time).unwrap();
    info!("[DB_LATENCY] request id {}: latency {} μs", req.param.get("id").unwrap(), db_duration.as_micros());

    //  Send the result to the invoker
    println!("{}", res);
}
