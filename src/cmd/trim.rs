use CliResult;
use config::{Config, Delimiter};
use util;

static USAGE: &'static str = "
Trims the fields of CSV

Usage:
    xsv trim [options] [<input>]

Common options:
    -h, --help             Display this message
    -o, --output <file>    Write output to <file> instead of stdout.
    -n, --no-headers       When set, the first row will not be interpreted
                           as headers. Namely, it will be reversed with the rest
                           of the rows. Otherwise, the first row will always
                           appear as the header row in the output.
    -d, --delimiter <arg>  The field delimiter for reading CSV data.
                           Must be a single character. (default: ,)
";

#[derive(Deserialize)]
struct Args {
    arg_input: Option<String>,
    flag_output: Option<String>,
    flag_no_headers: bool,
    flag_delimiter: Option<Delimiter>,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;
    let rconfig = Config::new(&args.arg_input)
        .delimiter(args.flag_delimiter)
        .no_headers(args.flag_no_headers);

    let mut rdr = rconfig.reader()?;

    let mut all = rdr.records().collect::<Result<Vec<_>, _>>()?;

    let mut wtr = Config::new(&args.flag_output).writer()?;


    if !rconfig.no_headers {
        let headers = rdr.headers()?;

        if !headers.is_empty() {
            let mut new_header = Vec::new();
            for col in headers.iter() {
                new_header.push(col.trim());
            }
            wtr.write_record(&new_header)?;
        }
    }

    for r in all.into_iter() {
        let mut new_row = Vec::new();
        for col in r.iter() {
            new_row.push(col.trim());
        }

        wtr.write_record(&new_row)?;
    }


    Ok(wtr.flush()?)
}
