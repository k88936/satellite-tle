use serde::{Serialize, Deserialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Satellite {
    pub name: String,
    
    // Line 1
    pub norad_id: u32,          // Satellite Catalog Number
    pub classification: char,   // 'U', 'C', 'S'
    pub int_designator: String, // International Designator (YYNNNPPP)
    pub epoch_year: u32,        // Last two digits of year
    pub epoch_day: f64,         // Day of the year and fractional portion
    pub first_derivative_mean_motion: f64, // Ballistic Coefficient
    pub second_derivative_mean_motion: f64, // Decimal point assumed
    pub bstar: f64,             // Drag term, decimal point assumed
    pub ephemeris_type: u8,
    pub element_set_number: u32,
    
    // Line 2
    pub inclination: f64,       // Degrees
    pub raan: f64,              // Right Ascension of the Ascending Node (Degrees)
    pub eccentricity: f64,      // Decimal point assumed
    pub argument_of_perigee: f64, // Degrees
    pub mean_anomaly: f64,      // Degrees
    pub mean_motion: f64,       // Revolutions per day
    pub revolution_number: u32, // Revolution number at epoch
}

impl Satellite {
    /// Parses a TLE string containing one or more satellites.
    pub fn parse_multiple(input: &str) -> Vec<Self> {
        let lines: Vec<&str> = input.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        
        let mut satellites = Vec::new();
        let mut i = 0;
        
        while i + 2 < lines.len() {
            if let Ok(sat) = Self::parse_from_lines(lines[i], lines[i+1], lines[i+2]) {
                satellites.push(sat);
            }
            i += 3;
        }
        satellites
    }

    fn parse_from_lines(name: &str, line1: &str, line2: &str) -> Result<Self, String> {
        if line1.len() < 68 { return Err(format!("Line 1 too short: {}", line1.len())); }
        if line2.len() < 68 { return Err(format!("Line 2 too short: {}", line2.len())); }

        // Helper function (nested but not closure capturing references weirdly)
        fn parse_val_str(line: &str, start: usize, end: usize) -> Result<&str, String> {
             if end > line.len() { return Err("Index out of bounds".to_string()); }
             Ok(line[start..end].trim())
        }
        
        let parse_u32 = |line: &str, start: usize, end: usize| -> Result<u32, String> {
            parse_val_str(line, start, end)?.parse::<u32>().map_err(|e| e.to_string())
        };
        
        let parse_f64 = |line: &str, start: usize, end: usize| -> Result<f64, String> {
            parse_val_str(line, start, end)?.parse::<f64>().map_err(|e| e.to_string())
        };

        // Helper for TLE special scientific notation: " 12345-5" -> 0.12345e-5
        let parse_tle_decimal = |line: &str, start: usize, end: usize| -> Result<f64, String> {
            let s = parse_val_str(line, start, end)?;
            if s.is_empty() { return Ok(0.0); }
            
            if s.contains('.') {
                return s.parse::<f64>().map_err(|e| e.to_string());
            }

            if s.len() < 2 { return Ok(0.0); } 
            
            let (mant_part, exp_part) = s.split_at(s.len() - 2);
            
            let exp_sign = &exp_part[0..1];
            let exp_digit = &exp_part[1..2];
            
            // Check if exponent sign is actually a digit (sometimes format is weird)
            // But standard TLE is strict: last char is exp digit, char before is sign (+/-).
            // Sometimes sign is space? " 0" -> +0.
            
            let exponent_str = format!("{}{}", 
                if exp_sign == "-" { "-" } else { "+" }, 
                exp_digit
            );
            
            let exponent: i32 = exponent_str.parse().map_err(|_| format!("Invalid exponent in {}", s))?;
            
            let mant_clean = mant_part.trim();
            if mant_clean.is_empty() { return Ok(0.0); }
            
            let mant_val: f64 = mant_clean.parse().map_err(|_| format!("Invalid mantissa in {}", s))?;
            
            // Count digits only (ignore sign)
            let digits = mant_clean.chars().filter(|c| c.is_digit(10)).count() as i32;
            
            let value = mant_val * 10f64.powi(-digits);
            let final_val = value * 10f64.powi(exponent);
            
            Ok(final_val)
        };

        // --- Line 1 ---
        // 1 65310U 25187F   25346.21683720  .00000042  00000+0  35694-3 0  9992
        let norad_id = parse_u32(line1, 2, 7)?;
        let classification = parse_val_str(line1, 7, 8)?.chars().next().unwrap_or('U');
        let int_designator = parse_val_str(line1, 9, 17)?.to_string();
        let epoch_year = parse_u32(line1, 18, 20)?;
        let epoch_day = parse_f64(line1, 20, 32)?;
        let first_derivative_mean_motion = parse_f64(line1, 33, 43)?;
        let second_derivative_mean_motion = parse_tle_decimal(line1, 44, 52)?;
        let bstar = parse_tle_decimal(line1, 53, 61)?;
        let ephemeris_type = parse_u32(line1, 62, 63)? as u8;
        let element_set_number = parse_u32(line1, 64, 68)?;

        // --- Line 2 ---
        let norad_id_2 = parse_u32(line2, 2, 7)?;
        if norad_id != norad_id_2 {
            return Err(format!("NORAD ID mismatch: {} vs {}", norad_id, norad_id_2));
        }

        let inclination = parse_f64(line2, 8, 16)?;
        let raan = parse_f64(line2, 17, 25)?;
        
        let ecc_str = parse_val_str(line2, 26, 33)?;
        let eccentricity = format!("0.{}", ecc_str).parse::<f64>().map_err(|e| e.to_string())?;
        
        let argument_of_perigee = parse_f64(line2, 34, 42)?;
        let mean_anomaly = parse_f64(line2, 43, 51)?;
        let mean_motion = parse_f64(line2, 52, 63)?;
        let revolution_number = parse_u32(line2, 63, 68)?;

        Ok(Satellite {
            name: name.to_string(),
            norad_id,
            classification,
            int_designator,
            epoch_year,
            epoch_day,
            first_derivative_mean_motion,
            second_derivative_mean_motion,
            bstar,
            ephemeris_type,
            element_set_number,
            inclination,
            raan,
            eccentricity,
            argument_of_perigee,
            mean_anomaly,
            mean_motion,
            revolution_number,
        })
    }
}

impl FromStr for Satellite {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.len() < 3 {
            return Err("Expected at least 3 lines for TLE".to_string());
        }

        Self::parse_from_lines(lines[0], lines[1], lines[2])
    }
}

impl fmt::Display for Satellite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;

        // --- Helper for TLE Scientific Notation ---
        // Converts value to SMMMMMSE format (e.g. 0.12345e-3 -> 12345-3)
        let format_tle_decimal = |val: f64| -> String {
            if val == 0.0 {
                return " 00000+0".to_string();
            }
            
            // Normalize: 0.xxxxx * 10^e
            let sci = format!("{:.4e}", val.abs()); 
            let parts: Vec<&str> = sci.split('e').collect();
            let base_val: f64 = parts[0].parse().unwrap();
            let exponent_val: i32 = parts[1].parse().unwrap();
            
            // Adjust to 0.xxxxx
            let final_exponent = exponent_val + 1;
            let final_mantissa = base_val / 10.0;
            
            // Round mantissa to 5 decimal places
            let mant_int = (final_mantissa * 100_000.0).round() as u64;
            
            let sign_num = if val < 0.0 { "-" } else { " " };
            let sign_exp = if final_exponent < 0 { "-" } else { "+" };
            
            format!("{}{:05}{}{}", sign_num, mant_int, sign_exp, final_exponent.abs() % 10)
        };

        // --- Line 1 ---
        let bstar_str = format_tle_decimal(self.bstar);
        let sec_deriv_str = format_tle_decimal(self.second_derivative_mean_motion);
        
        let first_deriv_str = format!("{:.8}", self.first_derivative_mean_motion).replace("0.", ".");
        let first_deriv_fmt = if self.first_derivative_mean_motion >= 0.0 {
            format!(" {}", first_deriv_str)
        } else {
            first_deriv_str
        };

        writeln!(f, "1 {:05}{} {:8} {:02}{:012.8} {:10} {} {} {} {:4}0",
            self.norad_id,
            self.classification,
            self.int_designator,
            self.epoch_year,
            self.epoch_day,
            first_deriv_fmt, 
            sec_deriv_str,
            bstar_str,
            self.ephemeris_type,
            self.element_set_number
        )?;

        // --- Line 2 ---
        let ecc_val = (self.eccentricity * 1e7).round() as u64;
        let ecc_str = format!("{:07}", ecc_val);
        
        write!(
            f,
            "2 {:05} {:8.4} {:8.4} {} {:8.4} {:8.4} {:11.8}{:5}0",
            self.norad_id,
            self.inclination,
            self.raan,
            &ecc_str[0..7.min(ecc_str.len())],
            self.argument_of_perigee,
            self.mean_anomaly,
            self.mean_motion,
            self.revolution_number
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TLE_RAW: &str = "
HULIANWANG DIGUI-78
1 65310U 25187F   25346.21683720  .00000042  00000+0  35694-3 0  9992
2 65310  50.0088 156.2785 0000258 123.5670 236.5227 13.29334203148340
HULIANWANG DIGUI-79
1 65311U 25187G   25346.27527670  .00000038  00000+0  34871-3 0  9991
2 65311  50.0017 156.5118 0000258 132.8838 227.2055 13.29335708148360
";

    #[test]
    fn test_parse() {
        let satellites = Satellite::parse_multiple(TLE_RAW);
        assert_eq!(satellites.len(), 2);

        let sat = &satellites[0];
        assert_eq!(sat.name, "HULIANWANG DIGUI-78");
        assert_eq!(sat.norad_id, 65310);
        assert_eq!(sat.classification, 'U');
        assert_eq!(sat.int_designator, "25187F");
        assert_eq!(sat.epoch_year, 25);
        assert_eq!(sat.epoch_day, 346.21683720);
        assert_eq!(sat.first_derivative_mean_motion, 0.00000042);
        assert_eq!(sat.second_derivative_mean_motion, 0.0); // 00000+0
        // 35694-3 -> 0.35694 * 10^-3 -> 0.00035694
        assert!((sat.bstar - 0.00035694).abs() < 1e-9);
        assert_eq!(sat.ephemeris_type, 0);
        assert_eq!(sat.element_set_number, 999); // 999 is element set, 2 is checksum
    }

    #[test]
    fn test_round_trip() {
        let satellites = Satellite::parse_multiple(TLE_RAW);
        let sat = &satellites[0];
        let tle = sat.to_string();
        
        let reparsed = Satellite::parse_multiple(&tle);
        assert_eq!(reparsed.len(), 1);
        let sat2 = &reparsed[0];
        
        assert_eq!(sat.norad_id, sat2.norad_id);
        
        // Check fuzzy float equality for BSTAR due to format conversion
        assert!((sat.bstar - sat2.bstar).abs() < 1e-8);
    }
}
