use clap::Parser;

const CSV_URL: &str = "./src/resources/timetable.csv";

#[derive(Parser)]
#[clap(name = "timetable", about = "A simple timetable manager")]
struct Opt {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
#[clap(about = "Subcommands")]
enum SubCommand {
    Add(Add),
    Print(Print),
    Clear(Clear),
}

#[derive(Parser)]
#[clap(name = "add", about = "Usage: add [DAY] [HOUR] [SUBJECT]")]
struct Add {
    /// Day of the week
    day: String,
    /// Time of the day (e.g. 09:00)
    time: String,
    /// The subject name
    subject: String,
}

#[derive(Parser)]
#[clap(name = "print", about = "Print the timetable")]
struct Print {}

#[derive(Parser)]
#[clap(name = "clear", about = "Clear the timetable")]
struct Clear {}

impl Add {
    fn run(&self) {
        let mut timetable = load_timetable(String::from(CSV_URL)).unwrap();

        // check if `time` includes a range (e.g. 09-12)
        if self.time.contains('-') {
            let times: Vec<&str> = self.time.split('-').collect();
            if times.len() == 2 {
                if let (Ok(start), Ok(end)) = (times[0].parse::<u8>(), times[1].parse::<u8>()) {
                    for hour in start..=end {
                        let formatted_time = format!("{}:00", hour);
                        add_timetable(
                            self.day.clone(),
                            formatted_time,
                            self.subject.clone(),
                            &mut timetable,
                        );
                    }
                }
            }
        } else {
            let formatted_time = if self.time.contains(':') {
                self.time.clone()
            } else {
                format!("{}:00", self.time)
            };

            add_timetable(
                self.day.clone(),
                formatted_time,
                self.subject.clone(),
                &mut timetable,
            );
        }

        let _ = save_timetable(String::from(CSV_URL), timetable);
    }
}

impl Print {
    fn run(&self) {
        print_timetable();
    }
}

impl Clear {
    fn run(&self) {
        clear_timetable();
        println!("Timetable cleared!");
    }
}

impl SubCommand {
    fn run(&self) {
        match self {
            SubCommand::Add(add) => add.run(),
            SubCommand::Print(print) => print.run(),
            SubCommand::Clear(clear) => clear.run(),
        }
    }
}

fn print_timetable() {
    let timetable_result = load_timetable(String::from(CSV_URL));

    match timetable_result {
        Ok(timetable) => {
            // determine the maximum width of each column
            let mut max_widths = vec![0; timetable[0].len()];
            for row in &timetable {
                for (i, cell) in row.iter().enumerate() {
                    max_widths[i] = max_widths[i].max(cell.len());
                }
            }

            let print_horizontal_border = || {
                for width in &max_widths {
                    print!("-");
                    print!("{}", "-".repeat(width + 2));
                }
                println!("-");
            };

            print_horizontal_border();

            // print each row with proper padding
            for (row_index, row) in timetable.iter().enumerate() {
                for (i, cell) in row.iter().enumerate() {
                    // adjust padding for each cell based on the maximum width of the column
                    print!("| {:width$} ", cell, width = max_widths[i]);
                }
                println!("|");

                if row_index == 0 {
                    print_horizontal_border();
                }
            }
            print_horizontal_border();
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn add_timetable(day: String, time: String, subject: String, timetable: &mut Vec<Vec<String>>) {
    let day_index = match day.as_str() {
        "ma" => 1,
        "ti" => 2,
        "ke" => 3,
        "to" => 4,
        "pe" => 5,
        "la" => 6,
        "su" => 7,
        _ => return,
    };

    for row in timetable.iter_mut() {
        if row[0] == time {
            if row.len() > day_index {
                row[day_index] = subject.clone();
                break;
            }
        }
    }
}

fn clear_timetable() {
    let mut timetable = vec![
        // 13 rows .repeat(8) times
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
        vec![""; 8],
    ];

    timetable[0] = vec![
        "",
        "Maanantai",
        "Tiistai",
        "Keskiviikko",
        "Torstai",
        "Perjantai",
        "Lauantai",
        "Sunnuntai",
    ]
    .iter()
    .map(|&s| s.into())
    .collect();

    let times = vec![
        "8:00", "9:00", "10:00", "11:00", "12:00", "13:00", "14:00", "15:00", "16:00", "17:00",
        "18:00", "19:00", "20:00",
    ];

    for (i, time) in times.iter().enumerate() {
        timetable[i + 1][0] = time.to_owned();
    }

    // fix mismatch types between timetable and save_timetable
    let timetable: Vec<Vec<String>> = timetable
        .iter()
        .map(|row| row.iter().map(|s| s.to_string()).collect())
        .collect();
    let _ = save_timetable(String::from(CSV_URL), timetable);
}

fn save_timetable(filename: String, timetable: Vec<Vec<String>>) -> Result<(), csv::Error> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_path(filename)?;

    for row in timetable {
        wtr.write_record(&row)?;
    }

    wtr.flush()?;
    Ok(())
}

fn load_timetable(filename: String) -> Result<Vec<Vec<String>>, csv::Error> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_path(filename)?;
    let mut timetable = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        timetable.push(row);
    }

    Ok(timetable)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        Print {}.run();
    } else {
        let args = Opt::parse();
        args.subcmd.run();
        Print {}.run();
    }
}
