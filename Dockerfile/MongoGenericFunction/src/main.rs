use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::sync::{Client, Database};
use log::{debug, info};
use std::{env};
use std::collections::HashMap;
use std::time::{ SystemTime};
use clap::Parser;

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

    #[clap(long, default_value = "" )]
    db_type: String,
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
// Convert the parameter of the the request to a Bson Document;
// Set update=true if the Document is used in an Update operation
fn from_param_to_doc(param : HashMap<String, String>, update: bool) -> Document {
    let mut doc: Document = Default::default();
    //	Split name, val
    for p in param{
        if p.0 == "_id" {
            doc.insert(p.0.as_str(), ObjectId::parse_str(p.1.as_str()).unwrap());
        }
        else {
            doc.insert(p.0.as_str(),p.1.as_str());
        }
    }
    if update{
        doc = doc!( "$set" : doc);
    }
    debug!("Document Created {}", doc);
    doc
}

// Execute the operation on DB
fn execute_operation(conn: Database, req: Request) -> String{
    //  Document generation
    let document = from_param_to_doc(req.param, false);
    debug!("Invoker | document to execute: {}", document);

    let answer = match req.op{
        Op::Create => {
            let start_time = SystemTime::now();
            let query_result =
                match conn.collection(req.table.as_str()).insert_one(document,None){
                    Ok(_) => String::from("Success"),
                    Err(_) =>String::from("Error")
                };
            let db_duration = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_duration.as_micros());

            query_result
        },
        Op::Read => {
            // Send back a Vec<Row> to keep the invoker independent from the data type
            let start_time = SystemTime::now();
            let query_result = conn.collection::<Document>(req.table.as_str()).find(document,None).unwrap();
            let db_duration = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_duration.as_micros());

            let mut query_string :String = String::new();
            for el in query_result{
                query_string = format!("{} {:?}",query_string, el);
            }
            query_string
        },
        Op::Update => {
            let update_doc = from_param_to_doc(req.param_to_up.unwrap(), true);

            let start_time = SystemTime::now();
            let query_result =
                match conn.collection::<HashMap<String, String>>(req.table.as_str()).update_one(document,update_doc, None){
                    Ok(_) => String::from("Success"),
                    Err(_) =>String::from("Error")
                };
            let db_duration = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_duration.as_micros());

            query_result
        },
        Op::Delete => {
            let start_time = SystemTime::now();
            let query_result =
                match conn.collection::<HashMap<String, String>>(req.table.as_str()).delete_one(document,None){
                    Ok(_) => String::from("Success"),
                    Err(_) =>String::from("Error")
                };
            let db_duration = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_duration.as_micros());

            query_result
        },
    };
    answer
}


fn main() {
    env_logger::init();
    let args = Args::parse();
    // DB parameters should be provided through environment or as at command invocation (the invoker provides the command args at launch)?
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("27017".to_string());
    let db = env::var("DB_NAME").unwrap_or("testDB".to_string());
    let url_db = format!("mongodb://{}:{}", address, port);

    let mut customer: HashMap<String, String> = HashMap::new();
    let mut customer_new: HashMap<String, String> = HashMap::new();

    // Connection to DB
    let client = Client::with_uri_str(url_db).unwrap();
    let mut conn = client.database(db.as_str());
    debug!("Invoker | Connected to DB: {:?}", conn);

    // added an id column (not auto increment) in the DBs so the client can add it manually
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
    if args.lastname_opt != "".to_string() {
        customer_new.insert("FIRSTNAME".to_string(), args.firstname_opt);
    }
    if args.lastname_opt != "".to_string() {
        customer_new.insert("LASTNAME".to_string(), args.lastname_opt);
    }

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

    // Operation execution
    let res  = execute_operation(conn, req);

    //  Send the result to the invoker
    println!("{}", res);
}