use std::error::Error;

#[derive(Debug, Clone)]
pub struct ParsedCliTable {
    pub fname: String,
    pub rows: Vec<CliTableRow>,
}

#[derive(Debug, Clone)]
pub struct CliTable {
    pub tables: Vec<ParsedCliTable>,
    pub regex_rules: Vec<CliTableRegexRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CliTableRegexRule {
    pub table_index: usize,
    pub row_index: usize,
    pub expanded_command: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CliTableRow {
    templates: Vec<String>,
    hostname: Option<String>,
    vendor: Option<String>,
    command: String,
}

impl ParsedCliTable {
    fn example(fname: &str) -> Result<Vec<CliTableRow>, Box<dyn Error>> {
        use std::io::BufReader;
        let file = std::fs::File::open(fname)?;
        let reader = BufReader::new(file);
        let mut rows: Vec<CliTableRow> = vec![];
        let mut rdr = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .has_headers(true)
            .delimiter(b',')
            .trim(csv::Trim::All)
            .from_reader(reader);
        println!("Reader");

        let headers: Vec<&str> = rdr.headers()?.into_iter().collect();
        println!("Headers: {:?}", &headers);

        if !headers.contains(&"Template") {
            return Err("No template".into());
        }
        if !headers.contains(&"Command") {
            return Err("No command".into());
        }

        let template_position = headers.iter().position(|x| *x == "Template").unwrap();
        let command_position = headers.iter().position(|x| *x == "Command").unwrap();
        let maybe_vendor_position = headers.iter().position(|x| *x == "Vendor");
        let maybe_hostname_position = headers.iter().position(|x| *x == "Hostname");

        for result in rdr.records() {
            let record = result?;
            let hostname: Option<String> =
                maybe_vendor_position.map(|vpos| record[vpos].to_string());
            let vendor: Option<String> =
                maybe_hostname_position.map(|hpos| record[hpos].to_string());
            let templates: Vec<String> = record[template_position]
                .split(":")
                .map(|x| x.to_string())
                .collect();
            let command = record[command_position].to_string();

            let row = CliTableRow {
                templates,
                hostname,
                vendor,
                command,
            };
            rows.push(row);
        }
        Ok(rows)
    }
    pub fn from_file(fname: &str) -> Self {
        println!("Loading cli table from {}", &fname);
        let rows = Self::example(fname).unwrap();
        ParsedCliTable {
            fname: fname.to_string(),
            rows,
        }
    }
}

impl CliTable {
    fn expand_string(input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        let chars: Vec<char> = input.chars().collect();
        let mut result = String::new();

        // Build the nested structure from left to right
        for (i, c) in chars.iter().enumerate() {
            // Add opening parenthesis and character
            result.push_str("(");
            result.push(*c);

            // For all characters except the last one, we'll need
            // to close their groups at the end
        }

        // Add closing parentheses and question marks
        result.push_str(&")?".repeat(chars.len()));

        result
    }

    fn expand_brackets(input: &str) -> String {
        let mut result = String::new();
        let mut current_pos = 0;

        while let Some(start) = input[current_pos..].find("[[") {
            // Add everything before the [[ to the result
            result.push_str(&input[current_pos..current_pos + start]);

            // Move position past the [[
            let content_start = current_pos + start + 2;

            // Look for matching ]]
            if let Some(end) = input[content_start..].find("]]") {
                let content = &input[content_start..content_start + end];
                let expanded = Self::expand_string(content);
                result.push_str(&expanded);
                current_pos = content_start + end + 2;
            } else {
                // No matching ]], treat [[ as literal
                result.push_str("[[");
                current_pos = content_start;
            }
        }

        // Add any remaining content
        result.push_str(&input[current_pos..]);
        result
    }

    pub fn from_file(fname: &str) -> Self {
        let parsed_cli_table = ParsedCliTable::from_file(fname);
        let tables = vec![parsed_cli_table];
        let mut regex_rules: Vec<CliTableRegexRule> = vec![];

        for (table_index, table) in tables.iter().enumerate() {
            for (row_index, row) in table.rows.iter().enumerate() {
                let expanded_command = Self::expand_brackets(&row.command);
                let rule = CliTableRegexRule {
                    table_index,
                    row_index,
                    expanded_command,
                };
                regex_rules.push(rule);
            }
        }
        CliTable {
            regex_rules,
            tables,
        }
    }
}
