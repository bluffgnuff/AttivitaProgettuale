package genericfunction;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.commons.cli.*;
import org.msgpack.jackson.dataformat.MessagePackFactory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.*;

public class JavaGenericFunctionWithFlag {

    public static void main(String[] args) {
        Map<String, String> env = System.getenv();
        String username = env.getOrDefault("NAME", "root");
        String password = env.getOrDefault("PASSWORD", "root");
        String address = env.getOrDefault("ADDRESS", "127.0.0.1");
        String port = env.getOrDefault("PORT", "3306");
        String db_name = env.getOrDefault("DB-NAME", "testDB");

        HashMap<String, String> customer = new HashMap<String, String>();
        HashMap<String, String> customer_new = new HashMap<String, String>();
        String table = "";
        String operation = "";

        CommandLine commandLine;
//		Controllo argomenti
        Options options = new Options();
        Option OptionTable = Option.builder("t")
                .required(false)
                .desc("The table in which work")
                .longOpt("table")
                .hasArg()
                .build();
        options.addOption(OptionTable);

        Option OptionOperation = Option.builder("o")
                .required(false)
                .desc("The operation to execute")
                .longOpt("operation")
                .hasArg()
                .build();
        options.addOption(OptionOperation);

        // added an id column (not auto increment) in the DBs so the client can add it manually
        Option OptionId = Option.builder("i")
                .required(false)
                .desc("The new entry ID")
                .longOpt("id")
                .hasArg()
                .build();
        options.addOption(OptionId);

        Option OptionFirstname = Option.builder("f")
                .required(false)
                .desc("The new entry firstname")
                .longOpt("firstname")
                .hasArg()
                .build();
        options.addOption(OptionFirstname);

        Option OptionLastname = Option.builder("l")
                .required(false)
                .desc("The new entry lastname")
                .longOpt("lastname")
                .hasArg()
                .build();
        options.addOption(OptionLastname);

        Option OptionLastname_opt = Option.builder("lo")
                .required(false)
                .desc("The the new lastname for the old entry")
                .longOpt("lastname_opt")
                .hasArg()
                .build();
        options.addOption(OptionLastname_opt);

        Option OptionFirstname_opt = Option.builder("fo")
                .required(false)
                .desc("The the new firstname for the old entry")
                .longOpt("firstname_opt")
                .hasArg()
                .build();
        options.addOption(OptionFirstname_opt);

        CommandLineParser commandLineParser = new DefaultParser();
        try {
            commandLine = commandLineParser.parse(options, args);
        } catch (ParseException e) {
            throw new RuntimeException(e);
        }

        if (commandLine.hasOption("table")) {
            table = commandLine.getOptionValue("table");
        }

        if (commandLine.hasOption("operation")) {
            operation = commandLine.getOptionValue("operation");
        }

        if (commandLine.hasOption("id")) {
            customer.put("id", commandLine.getOptionValue("id"));
        }

        if (commandLine.hasOption("firstname")) {
            customer.put("firstname", commandLine.getOptionValue("firstname"));
        }

        if (commandLine.hasOption("lastname")) {
            customer.put("lastname", commandLine.getOptionValue("lastname"));
        }

        if (commandLine.hasOption("lastname_opt")) {
            customer_new.put("lastname_opt", commandLine.getOptionValue("lastname_opt"));
        }

        if (commandLine.hasOption("firstname_opt")) {
            customer_new.put("firstname_opt", commandLine.getOptionValue("firstname_opt"));
        }

        //GenericFunction.Request { op: Create, table: "Customers", param: {"id": "1000000-3", "LASTNAME": "Rossi", "FIRSTNAME": "Giuseppe"}, param_to_up: Some({}) }
//        Request req = new Request(Op.valueOf(operation), table, customer, Optional.empty());

        ObjectMapper mapper = new ObjectMapper(new MessagePackFactory());
        int obj = 42;


//"[148, 166, 67, 114, 101, 97, 116, 101, 169, 67, 117, 115, 116, 111, 109, 101, 114, 115, 131, 169, 70, 73, 82, 83, 84, 78, 65, 77, 69, 168, 71, 105, 117, 115, 101, 112, 112, 101, 162, 105, 100, 169, 49, 48, 48, 48, 48, 48, 48, 45, 50, 168, 76, 65, 83, 84, 78, 65, 77, 69, 165, 82, 111, 115, 115, 105, 128]"
        //  GenericFunction.Request packaging
        byte [] req_pack;

        try {
            req_pack = mapper.writeValueAsBytes(obj);
            StringBuilder req_builder = new StringBuilder();

            req_builder.append("[");
            Boolean first = true;
            for(byte el : req_pack) {
                if (first){
                    first=false;
                }
                else{
                    req_builder.append(", ");
                }
                req_builder.append(el);
            }
            req_builder.append("]");

            //  Send req through stdout
            System.out.println(req_builder.toString());
        } catch (JsonProcessingException e) {
            throw new RuntimeException(e);
        }

        //  Receive the answer through stdin
        BufferedReader br = new BufferedReader(new InputStreamReader(System.in));
        String result;
        try {
            result = br.readLine();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
//        debug!("Data received: {:?}",result );

        //  Deserialize
//         req_serialized
        try {
            String gnagna = mapper.readValue(result, new TypeReference<String>(){});
        } catch (JsonProcessingException e) {
            throw new RuntimeException(e);
        }

//        if ( operation == "Read") {
//            let req_serialized:Vec<u8> = result.split(", ").map(|x| x.parse().unwrap()).collect();
//            debug!("Serialized answer {:?}", req_serialized);
//            let req :Vec<String> = rmp_serde::from_read_ref(&req_serialized).unwrap();
//
//            if (args.db_type != "CouchDB" ){
//                let mut des_answ= String::new();
//                for el in req {
//                    des_answ = format!("{} {:?}", des_answ, el);
//                }
//                println!("{}", des_answ);
//            }
//            else { // case CouchDB
//                    println!("{:?}", req);
//                }
//            }
//        else{
//                println!("{:?}", result);
//        }
    }
}
