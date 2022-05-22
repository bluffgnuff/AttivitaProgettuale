const axios = require('axios');

const axiosUrlComposer = (host, port) => { return 'http://' + host + ':' + port }
const { v4: uuidv4 } = require('uuid');
const DEFAULTDB = process.env.DEFAULTDB === undefined ? "poi" : process.env.DEFAULTDB;
//BUG correct defaultdb


exports.createDB= function createDB(host, port, db = DEFAULTDB, partition = undefined) {
  let obj = {}

  obj.partition = partition;
  obj.port = port;
  obj.host = host;
  obj.baseurl = axiosUrlComposer(host, port)
  obj.db = db
  obj.urldb = partition === undefined ? obj.baseurl + '/' + db + '/' : obj.baseurl + '/' + db + '/_partition/' + partition + "/"

  return obj;
}

exports.getAllDocs= function getAllDocs(db = DEFAULTDB) {
  var url = db.urldb + '_all_docs'
  return axios.get(url).then(resp => resp.data)
    .catch(err => { console.error(err); return [] })
}

function getNodeMango(param, db = DEFAULTDB) {
  var url = db.urldb + '_find'
  var data = JSON.stringify(param);
  var config = {
    method: 'post',
    url,
    headers: {
      'Content-Type': 'application/json'
    },
    data,
  };
  var res = axios(config).then(resp => resp.data.docs)
    .catch(err => { console.error(err); return [] })

  return res
}

exports.getNodeMango=getNodeMango;

//TODO USEME
addDoc= function addDoc(param, db = DEFAULTDB) {
  var url = db.urldb + uuidv4()
  var data = JSON.stringify(param);
  var config = {
    method: 'put',
    url,
    headers: {
      'Content-Type': 'application/json'
    },
    data,
  };
  var res = axios(config)
    .catch(err => { console.log(err) })

  return res
}

exports.getFirstNodePartition= function(partition,db=DEFAULTDB,policy=basicPolicy){
  let nodes=getNodesPartition(partition,db);
  return getNodesPartition[basicPolicy(nodes)]
}

function getNodesPartition(partition,db=DEFAULTDB){

}

exports.getNodesPartition=getNodesPartition;

function basicPolicy(nodes){
  let index=Math.round(Math.random()*nodes.length);
  return nodes[index];
}


// return 
//@Array<{_id,hostname,password}
 function getDocsCastedFileds(db, jsonFileds) {
  let param= {
    "selector": {
       "_id": {
          "$gt": null
       }
    },
    "fields": [
       "_id",
    ]
  }; 
  param.fields=param.fields.concat(jsonFileds);

  return getNodeMango(param, db)
}

var id = process.argv[4];
var create = {op: 1, table: "Customers", param: {"firstname": "Luca","lastname":"Rossi", "id":id }, param_to_up: null };

//var read = {op: 2, table: "Customers", param: {"firstname": "Luca","lastname":"Rossi", "id":"id-0000" }, param_to_up: null };

//TODO CHANGE ME !!!!
var host = "192.168.10.206";
var port = "5984";
var db_name = "testdb";
var db = this.createDB(host, port, db_name);

//DB operatations 
var before = new Date().getTime();
var res = addDoc(create.param, db);
//var par_to_find = {"selector": {"id": {"$eq": "id-0000"}}}
//var res = getNodeMango(par_to_find, db);
var after = new Date().getTime();
//latenza 
var latecy = (after - before)*1000;

//print info level
console.info("[DB_LATENCY] "+latecy);

//Print result
res.then( data => console.log(data));
