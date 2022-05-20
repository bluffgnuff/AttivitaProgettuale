const mongoose = require('mongoose');

var host = "192.168.10.206";
var port = "27017";
var db_name = "testDB";
mongoose.connect('mongodb://'+ host +':' + port+'/'+ db_name);

var id = process.argv[3];

const Customers = mongoose.model('Customers', { firstname: String , lastname: String, id:String });
const kitty = new Customers({ firstname: 'Mario', lastname: 'Rossi', id: id });

var before = new Date().getTime();
kitty.save().then((res) => { 
    
    var after = new Date().getTime();
    //latenza 
    var latecy = after - before;
    //print info level
    console.info(latecy);
    console.log(res);
    process.exit(0);
    });
