use std::fs;
use std::path::Path;
use gcode_nom::{parse_gcode, Gcode, Argument};

#[derive(Debug)]
pub struct FilamentEstimator {
    diameter_mm: f64,
    density_g_cm3: f64,
    cost_per_kg: f64,
    
    absolute_extrusion: bool,
    total_extruded_mm: f64,
    last_e_value: f64,
}

impl FilamentEstimator {
    pub fn new(diameter_mm: f64, density_g_cm3: f64, cost_per_kg: f64) -> Self {
        Self {
            diameter_mm,
            density_g_cm3,
            cost_per_kg,
            absolute_extrusion: true, // M82 default
            total_extruded_mm: 0.0,
            last_e_value: 0.0,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        
        // Parse the entire file using gcode-nom
        // parse_gcode returns a list of parsed Gcode statements
        let (_, gcodes) = parse_gcode(&content)
            .map_err(|e| format!("Failed to parse G-code: {:?}", e))?;

        for gcode in gcodes {
            self.process_command(&gcode);
        }

        self.print_summary();
        Ok(())
    }

    fn process_command(&mut self, gcode: &Gcode) {
        // gcode-nom breaks commands down by letter and arguments
        match gcode.letter() {
            // Handle M-codes (Configuration / Modes)
            'M' => {
                if gcode.code() == 82 {
                    self.absolute_extrusion = true;
                } else if gcode.code() == 83 {
                    self.absolute_extrusion = false;
                    self.last_e_value = 0.0;
                }
            }
            // Handle G-codes (Motion / Resets)
            'G' => {
                let code = gcode.code();
                
                // G92 E0 resets the extruder axis position
                if code == 92 {
                    if let Some(e) = self.get_argument(gcode, 'E') {
                        self.last_e_value = e;
                    }
                }
                
                // G0 / G1 Linear Moves
                if code == 0 || code == 1 {
                    if let Some(e_val) = self.get_argument(gcode, 'E') {
                        if self.absolute_extrusion {
                            let delta = e_val - self.last_e_value;
                            if delta > 0.0 {
                                self.total_extruded_mm += delta;
                            }
                            self.last_e_value = e_val;
                        } else {
                            if e_val > 0.0 {
                                self.total_extruded_mm += e_val;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn get_argument(&self, gcode: &Gcode, target_letter: char) -> Option<f64> {
        for arg in gcode.arguments() {
            if arg.letter() == target_letter {
                // Extract float value from argument
                return Some(arg.value());
            }
        }
        None
    }

    fn print_summary(&self) {
        let length_m = self.total_extruded_mm / 1000.0;
        let r_cm = (self.diameter_mm / 2.0) / 10.0;
        let length_cm = self.total_extruded_mm / 10.0;
        
        let volume_cm3 = std::f64::consts::PI * r_cm.powi(2) * length_cm;
        let weight_g = volume_cm3 * self.density_g_cm3;
        let cost = (weight_g / 1000.0) * self.cost_per_kg;

        println!("--- Filament Usage Estimation ---");
        println!("Total Length:   {:.2} meters", length_m);
        println!("Total Volume:   {:.2} cm³", volume_cm3);
        println!("Estimated Mass: {:.2} grams", weight_g);
        println!("Estimated Cost: ${:.2}", cost);
    }
}