use fancy_regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParsedCliTable {
    pub fname: String,
    pub rows: Vec<CliTableRow>,
}

#[derive(Debug, Clone)]
pub struct CliTable {
    pub tables: Vec<ParsedCliTable>,
    pub platform_regex_rules: HashMap<String, Vec<CliTableRegexRule>>,
}

#[derive(Debug, Clone)]
pub struct CliTableRegexRule {
    pub table_index: usize,
    pub row_index: usize,
    pub command_regex: Regex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CliTableRow {
    templates: Vec<String>,
    hostname: Option<String>,
    platform: Option<String>,
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
        let maybe_platform_position = headers.iter().position(|x| *x == "Platform");
        let maybe_hostname_position = headers.iter().position(|x| *x == "Hostname");

        for result in rdr.records() {
            let record = result?;
            let platform: Option<String> =
                maybe_platform_position.map(|ppos| record[ppos].to_string());
            let hostname: Option<String> =
                maybe_hostname_position.map(|hpos| record[hpos].to_string());
            let templates: Vec<String> = record[template_position]
                .split(":")
                .map(|x| x.to_string())
                .collect();
            let command = record[command_position].to_string();

            let row = CliTableRow {
                templates,
                hostname,
                platform,
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

    fn get_directory(filename: &str) -> Option<String> {
        let path = Path::new(filename);
        path.parent().map(|p| p.to_string_lossy().into_owned())
    }

    pub fn get_template_for_command(
        &self,
        platform: &str,
        cmd: &str,
    ) -> Option<(String, CliTableRow)> {
        let plat_regex_list = self
            .platform_regex_rules
            .get(platform)
            .expect("Could not find platform");
        for rule in plat_regex_list {
            if rule.command_regex.is_match(cmd).expect("Fancy regex ok?") {
                let row = self.tables[rule.table_index].rows[rule.row_index].clone();
                let fname = &self.tables[rule.table_index].fname;
                if let Some(fdir) = Self::get_directory(fname) {
                    return Some((fdir, row));
                }
            }
        }
        None
    }

    pub fn from_file(fname: &str) -> Self {
        let parsed_cli_table = ParsedCliTable::from_file(fname);
        let tables = vec![parsed_cli_table];
        let mut platform_regex_rules: HashMap<String, Vec<CliTableRegexRule>> = Default::default();

        for (table_index, table) in tables.iter().enumerate() {
            for (row_index, row) in table.rows.iter().enumerate() {
                let expanded_command = Self::expand_brackets(&row.command);
                let command_regex = Regex::new(&expanded_command).unwrap();

                let rule = CliTableRegexRule {
                    table_index,
                    row_index,
                    command_regex,
                };
                let no_platform = format!("no-platform");
                let platform_name: &str = row.platform.as_ref().unwrap_or(&no_platform);
                platform_regex_rules
                    .entry(platform_name.into())
                    .or_insert_with(Vec::new)
                    .push(rule);
            }
        }
        CliTable {
            platform_regex_rules,
            tables,
        }
    }
}
