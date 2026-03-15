use std::fs;
use std::path::Path;
use satelite_tle::Satellite;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = Path::new("examples/beidou.txt");
    println!("Reading from: {:?}", file_path);
    
    if !file_path.exists() {
        eprintln!("Error: Could not find gp_beidou.txt");
        return Ok(());
    }

    let content = fs::read_to_string(file_path)?;
    let satellites = Satellite::parse_multiple(&content);

    println!("Found {} satellites.", satellites.len());
    println!("{:-<70}", "");
    println!("{:<25} | {:<10} | {:<12} | {:<12}", "Name", "NORAD ID", "Inclination", "Mean Motion");
    println!("{:-<70}", "");

    for sat in satellites.iter().take(10) {
        println!("{:<25} | {:<10} | {:<12.4} | {:<12.8}", 
            sat.name.trim(), 
            sat.norad_id, 
            sat.inclination,
            sat.mean_motion
        );
    }
    
    if satellites.len() > 10 {
        println!("... and {} more", satellites.len() - 10);
    }

    Ok(())
}
