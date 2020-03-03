use csv;
use regex::bytes::RegexBuilder;

use CliResult;
use config::{Config, Delimiter};
use select::SelectColumns;
use util;

static USAGE: &'static str = "
Fills any fields matched by the <pattern> with value of <fill-column>

The regex is only applied to the selected column. Multiple columns may be given.
See 'xsv select --help' for syntax.

Usage:
    xsv fill [options] <select> <regex> <fill-column> [<input>]
    xsv fill --help

Example:
    Replace any empty values in Column1 with values from Column2:
        xsv fill Column1 '^$' Column2 file.csv

search options:
    -i, --ignore-case      Case insensitive search. This is equivalent to
                           prefixing the regex with '(?i)'.
    -s, --select <arg>     Select the columns to search. See 'xsv select -h'
                           for the full syntax.
    -v, --invert-match     Select only rows that did not match

Common options:
    -h, --help             Display this message
    -o, --output <file>    Write output to <file> instead of stdout.
    -n, --no-headers       When set, the first row will not be interpreted
                           as headers. (i.e., They are not searched, analyzed,
                           sliced, etc.)
    -d, --delimiter <arg>  The field delimiter for reading CSV data.
                           Must be a single character. (default: ,)
";

#[derive(Deserialize)]
struct Args {
    arg_input: Option<String>,
    arg_fill_column: SelectColumns,
    arg_regex: String,
    arg_select: SelectColumns,
    flag_output: Option<String>,
    flag_no_headers: bool,
    flag_delimiter: Option<Delimiter>,
    flag_invert_match: bool,
    flag_ignore_case: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;
    let pattern = RegexBuilder::new(&*args.arg_regex)
        .case_insensitive(args.flag_ignore_case)
        .build()?;
    let rconfig = Config::new(&args.arg_input)
        .delimiter(args.flag_delimiter)
        .no_headers(args.flag_no_headers)
        .select(args.arg_select);

    let mut rdr = rconfig.reader()?;
    let mut wtr = Config::new(&args.flag_output).writer()?;

    let headers = rdr.byte_headers()?.clone();
    let sel = rconfig.selection(&headers)?;

    let rep_sel = args.arg_fill_column.selection(&headers, !args.flag_no_headers)?;

    if !rconfig.no_headers {
        wtr.write_record(&headers)?;
    }
    let mut record = csv::ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
        let mut m = sel.select(&record).any(|f| pattern.is_match(f));

        // While multiple replacement columns may be theoretically given,
        // we only consider the first one
        let replacement = rep_sel.select(&record).next();
        let mut new_record = csv::ByteRecord::new();

        if args.flag_invert_match {
            m = !m;
        }

        for (index, col) in record.iter().enumerate() {
            if m && sel.contains(&index) {
                match replacement{
                    Some(r)  => new_record.push_field(&r),
                    None => new_record.push_field(b""),
                };
            } else {
                new_record.push_field(col);
            }
        }
        wtr.write_byte_record(&new_record)?;
    }
    Ok(wtr.flush()?)
}
