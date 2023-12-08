use clap::{Command, Arg,};
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use tempfile;

fn main() {
    let cli = Command::new("themisto-remapper")
        .about("Compresses the color range of Themisto output by removing all colors with less than a given number of total hits")
        .author("Jarno N. Alanko <alanko.jarno@gmail.com>")
        .arg_required_else_help(true)
        .arg(Arg::new("input")
            .help("Themisto pseudoalignment file")
            .required(true)
            .short('i')
            .long("input")
            .value_parser(clap::value_parser!(std::path::PathBuf)))
        .arg(Arg::new("output")
            .help("Output pseudoalignment file")
            .required(true)
            .short('o')
            .long("output")
            .value_parser(clap::value_parser!(std::path::PathBuf)))
        .arg(Arg::new("mapping-file")
            .help("Mapping output file with pairs of new and old colors, one per line, separated by a tab.")
            .required(true)
            .short('m')
            .long("mapping-file")
            .value_parser(clap::value_parser!(std::path::PathBuf)))
        .arg(Arg::new("min-hits")
            .help("Minimum number of hits for a color to be included in the output")
            .required(true)
            .short('n')
            .long("min-hits")
            .value_parser(clap::value_parser!(usize)));


    let matches = cli.get_matches();    

    let inputfile: &std::path::PathBuf = matches.get_one("input").unwrap();
    let outputfile: &std::path::PathBuf = matches.get_one("output").unwrap();
    let mappingfile: &std::path::PathBuf = matches.get_one("mapping-file").unwrap();
    let min_hits: usize = *matches.get_one("min-hits").unwrap();

    run(inputfile, outputfile, mappingfile, min_hits);

}

fn run(inputfile: &std::path::PathBuf, outputfile: &std::path::PathBuf, mappingfile: &std::path::PathBuf, min_hits: usize) {

    let mut reader = std::io::BufReader::new(std::fs::File::open(inputfile).unwrap()); 
    let mut line = String::new();

    // Hash map of color -> count
    let mut counts = std::collections::HashMap::<usize, usize>::new();

    // Get counts of colors
    let mut max_color = 0_usize;
    while reader.read_line(&mut line).unwrap() > 0 {
        let mut tokens = line.trim().split(' ');

        let _= tokens.next(); // Read id
        for color in tokens {
            let color = color.parse::<usize>().unwrap();
            max_color = std::cmp::max(max_color, color);
            *(counts.entry(color).or_insert(0)) += 1;
        }

        // Clear the line buffer
        line.clear();
    } 

    // Get all keys which has a count of at least min_hits
    let mut kept_colors = counts.into_iter().filter(|(_, count)| *count >= min_hits).map(|(color, _)| color).collect::<Vec<usize>>();
    kept_colors.sort();


    // Build the mapping and write the mapping to file
    let mut mapping_writer = std::io::BufWriter::new(std::fs::File::create(mappingfile).unwrap());
    let mut old_to_new = Vec::<Option<usize>>::new();
    old_to_new.resize(max_color+1, None); // old_to_new[i] = name of new color for color i.
    for (new_color, old_color) in kept_colors.iter().enumerate() {
        old_to_new[*old_color] = Some(new_color);
        writeln!(mapping_writer, "{}\t{}", new_color, old_color).unwrap(); // new to old
    }

    // Write the new pseudoalignment file
    let mut reader = std::io::BufReader::new(std::fs::File::open(inputfile).unwrap());
    let mut writer = std::io::BufWriter::new(std::fs::File::create(outputfile).unwrap());
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        let mut tokens = line.trim().split(' ');

        let read_id = tokens.next().unwrap();
        write!(writer, "{}", read_id).unwrap();
        for color in tokens {
            let old_color = color.parse::<usize>().unwrap();
            if let Some(new_color) = old_to_new[old_color] {
                write!(writer, " {}", new_color).unwrap();
            }
        }
        writeln!(writer).unwrap();

        // Clear the line buffer
        line.clear();
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn basic_test(){
        // {0 5 7} {5 0 7 8 2} {0 1 2 3 4 5}
        // Min hits = 2
        // Should keep 0, 2, 5, 7

        let input_data = b"1 0 5 7\n2 5 0 7 8 2\n0 0 1 2 3 4 5\n";
        let mut input_file = tempfile::NamedTempFile::new().unwrap();
        input_file.write_all(input_data).unwrap();
        input_file.flush().unwrap();

        let mut output_file = tempfile::NamedTempFile::new().unwrap();
        let mut mapping_file = tempfile::NamedTempFile::new().unwrap();
        let min_hits = 2;
        run(&input_file.path().to_path_buf(), &output_file.path().to_path_buf(), &mapping_file.path().to_path_buf(), min_hits);

        // Read all btes from output files
        let mut output_bytes = Vec::<u8>::new();
        output_file.read_to_end(&mut output_bytes).unwrap();
        let mut mapping_bytes = Vec::<u8>::new();
        mapping_file.read_to_end(&mut mapping_bytes).unwrap();

        eprintln!("Output bytes: {:?}", String::from_utf8(output_bytes.clone()).unwrap());
        eprintln!("Mapping bytes: {:?}", String::from_utf8(mapping_bytes.clone()).unwrap());

        assert_eq!(output_bytes, b"1 0 2 3\n2 2 0 3 1\n0 0 1 2\n");
        assert_eq!(mapping_bytes, format!("{}\t{}\n{}\t{}\n{}\t{}\n{}\t{}\n", 0,0, 1,2, 2,5, 3,7).as_bytes());
    }
}