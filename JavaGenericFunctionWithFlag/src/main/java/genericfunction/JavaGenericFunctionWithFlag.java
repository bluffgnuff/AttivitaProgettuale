package genericfunction;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.commons.cli.*;
import org.msgpack.jackson.dataformat.MessagePackFactory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

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
		String operation = "Read";

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

		Request req = new Request(Op.valueOf(operation), table, customer, null);

		ObjectMapper mapper = new ObjectMapper(new MessagePackFactory());
		Map<String, Object> obj = new HashMap<String, Object>();
		obj.put("Request", req);
		obj.put("op", req.getOp());
		obj.put("table", req.getTable());
		obj.put("param", req.getParam());
		obj.put("param_to_up", req.getParam_to_up());

		try {
			byte[] bytes = mapper.writeValueAsBytes(obj);

			Boolean first = true;
			StringBuilder req_builder = new StringBuilder();

			req_builder.append("[");
			for (Byte el : bytes) {
				if (first) {
					first = false;
				} else {
					req_builder.append(", ");
				}
				int uint8 = el & 0xFF;
				req_builder.append(uint8);
			}
			req_builder.append("]");

			//  Send req through stdout
			System.out.println(req_builder);

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

		if (req.getOp().equals(Op.Read)) {
			//  Deserialize
			try {
				String[] byteValues = result.split(", ");
				byte[] bs = new byte[byteValues.length];

				for (int i=0; i<bs.length; i++) {
					bs[i] = (byte) Integer.parseInt(byteValues[i].trim());
				}

				TypeReference<List<Object>> typeReference = new TypeReference<List<Object>>(){};
				List<Object> result_deserialized = mapper.readValue(bs, typeReference);

				System.out.println(result_deserialized);
			} catch (IOException e) {
				throw new RuntimeException(e);
			}
		} else {
			System.out.println(result);
		}
	}
}

