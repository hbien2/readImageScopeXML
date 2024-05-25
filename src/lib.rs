use std::{error, path, fs::File, io::BufReader, ffi::OsString};
use xml::reader::{EventReader, XmlEvent};

pub fn run(search_path: &path::Path) -> Result<(), Box<dyn error::Error>> {
    // Iterate through list of files in search path looking for XML files only
    for entry in search_path.read_dir().expect("Invalid search path").filter(|dirent| {
        dirent.as_ref().is_ok_and(|d|  {
            return d.path().as_path().extension().is_some_and(|e| e.to_ascii_lowercase()==OsString::from("xml"));
        })
    }) {        
        let filepath = entry?.path(); // Since this is filtered, all values of the entry iterator should have valid path() so safe to use unwrap()
        //dbg!(&filepath);
        let file = File::open(&filepath);           
        match file {
            Ok(file)=> {
                // Open file for reading
                let file = BufReader::new(file);
                // Read using XML parser
                let parser = EventReader::new(file);
                // Flag true when inside annotation Type="3" block
                let mut in_annotation_type = false;
                // Flag true when inside Attributes block
                let mut in_attributes = false;
                                
                // Examine each XML element event (start/stop)
                'elementscan: for element in parser {
                    match element {
                        Ok(XmlEvent::StartElement { name, attributes, namespace: _ }) => {
                            match name.local_name.as_str() {
                                    "Annotation" => {
                                        //dbg!(&name, &attributes);
                                        // Search for attribute name "Type" and value="3"
                                        for attrib in attributes.into_iter().filter(|v| v.name.local_name=="Type") {                                        
                                            // Find type 4 annotation
                                            if attrib.value=="3" {
                                                in_annotation_type=true;
                                            }                                            
                                        }
                                    },
                                    "Attributes" => {
                                        in_attributes=true;
                                    },
                                    "Attribute" => {
                                        // Search only when inside Attributes block within a Type 3 Annotation block
                                        if in_annotation_type && in_attributes {
                                            // Run through all the attributes; we expect the Name attribute BEFORE the Value attribute
                                            // Flag TRUE when Name=="Positivity = NPositive/NTotal" attribute is encountered in the list of attributes
                                            let mut in_positivity=false;
                                            for attrib in attributes {                                              
                                                if attrib.name.local_name=="Name" && attrib.value=="Positivity = NPositive/NTotal" {                                                    
                                                    in_positivity=true;
                                                    // Move to next attribute
                                                    continue;
                                                }
                                                if in_positivity && attrib.name.local_name=="Value" {
                                                    // Convert positivity value to numeric (float32)
                                                    let positivity = attrib.value.parse::<f32>();
                                                    if let Ok(positivity) = positivity {
                                                        // Report on the Positivity value
                                                        println!("{:?},{:?}", filepath.file_name().expect("Unexpected error extracting filename after opening file"), positivity);
                                                        // Now skip to next file
                                                        continue 'elementscan;
                                                    } else {
                                                        eprintln!("In file {:?}, unable to convert Positivity value {:?} to type float32", &filepath, attrib.value.as_str());
                                                    }                                                    
                                                    // Exit loop regardless as we are done processing this <Attribute> element
                                                    break;
                                                }
                                            }
                                        }                                        
                                    },
                                    &_ => {
                                        // Ignore all others
                                    },
                                }                                
                            }
                            Ok(XmlEvent::EndElement { name}) => {
                                match name.local_name.as_str() {
                                    // Exit from annotation type block
                                    "Annotation" => {in_annotation_type=false;},
                                    // Exit from attributes block
                                    "Attributes" => {in_attributes=false;},
                                    // Ignore all others
                                    &_ => {}
                                }                                                                
                            }
                            Err(e) => {
                                eprintln!("Error: {e}");
                                break;
                            }
                            // Ignore everything else
                            _ => {}
                        }
                    }
                },
                Err(error) => eprintln!("Error opening file: {:?}", error),
        } 
    } 
    return Ok(());
}
