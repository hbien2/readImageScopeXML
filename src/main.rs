use std::{env, path, error};

fn main() -> Result<(), Box<dyn error::Error>> {
    // Start by collecting command line arguments
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    // Default is use executable folder as search path
    let mut search_path = path::Path::new(&args[0]).parent().expect("Parent folder of executable should always be available and valid");
    // If an argument is specified, use that directly instead
    if args.len()>=2 {
        // Create a search Path from provided argument directly
        search_path = path::Path::new(&args[1]);
    } 
    
    dbg!(&search_path);

    return read_image_scope_xml::run(search_path);        
}
