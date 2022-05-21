const mongoose = require('mongoose');

var host = "192.168.10.206";
var port = "27017";
var db_name = "testDB";
var id = process.argv[4];

mongoose.connect('mongodb://'+ host +':' + port+'/'+ db_name);

const Customers = mongoose.model('CustomersNoConn', { firstname: String , lastname: String, id:String });
const kitty = new Customers({ firstname: 'Mario', lastname: 'Rossi', id: id });

var before = new Date().getTime();

Customers.find({id: 'id-00001'}).then((res) => { 
    
    var after = new Date().getTime();
    //latenza 
    var latecy = (after - before)*1000;
    //print info level
    console.info("[DB_LATENCY] "+latecy);
    console.log(res);
    process.exit(0);
    });
