package mongo.function;

import com.mongodb.MongoClient;
import com.mongodb.client.FindIterable;
import com.mongodb.client.MongoCollection;
import com.mongodb.client.MongoDatabase;
import org.apache.commons.cli.*;
import org.bson.Document;

import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import java.util.logging.Logger;

//--table Customers --operation Read --id idProva2 --firstname NomeProva --lastname CognomeProva
public class JavaMongoGenericFunction {

	public static void main(String[] args) {
		Map<String, String> env = System.getenv();
		String username = env.getOrDefault("NAME", "root");
		String password = env.getOrDefault("PASSWORD", "root");
		String address = env.getOrDefault("ADDRESS", "127.0.0.1");
		String port = env.getOrDefault("PORT", "27017");
		String db_name = env.getOrDefault("DB-NAME", "testDB");

		HashMap<String, String> customer = new HashMap<String, String>();
		HashMap<String, String> customer_new = new HashMap<String, String>();
		String table = "";
		String operation = "";

		// Create a Logger
		Logger logger
				= Logger.getLogger(
				JavaMongoGenericFunction.class.getName());
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
//        if (commandLine.hasOption("_rev")){
//        }

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

//       Connection to Mongo DB
		try {
			MongoClient mongoClient = new MongoClient(address, Integer.parseInt(port));
			MongoDatabase database = mongoClient.getDatabase(db_name);

//          TODO Extend to UPDATE
			MongoCollection<Document> collection = database.getCollection(table);
			Document document = new Document();
			document.putAll(customer);

			if (operation.equals("Create")) {
				long before = TimeUnit.MILLISECONDS.toMicros(System.nanoTime());
				collection.insertOne(document);
				long after = TimeUnit.MILLISECONDS.toMicros(System.nanoTime());
				long latencyMicros = (after - before) / 1000;

				logger.info("[DB_LATENCY] latency " + latencyMicros + " μs");

				String result = "FINITO!!";
				System.out.println(result);
			} else {
				long before = TimeUnit.MILLISECONDS.toMicros(System.nanoTime());
				FindIterable<Document> result = collection.find(document);
				long after = TimeUnit.MILLISECONDS.toMicros(System.nanoTime());
				long latencyMicros = (after - before) / 1000;

				logger.info("[DB_LATENCY] latency " + latencyMicros + " μs");

				for (var doc : result) {
					System.out.println(doc);
				}

			}
		} catch (NumberFormatException e) {
			throw new RuntimeException(e);
		}
	}
}