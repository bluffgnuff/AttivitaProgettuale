var msgpck= require ('@msgpack/msgpack');
const prompt = require ('prompt');
prompt.start();

var id = process.argv[3];
var create = {op: 1, table: "Customers", param: {"firstname": "Luca","lastname":"Rossi", "id": id }, param_to_up: null };

var encoded = msgpck.encode(create);
console.log("[" + encoded.toString() + "]");

const readline = require('readline');
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

var result;

prompt.get(['result'], function(err, result){console.log(result);} );
