use std::{error, path, ffi::OsString};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::collections::HashMap;

/// Information we wish to collect about a region
#[derive(Debug)]
struct RegionInfo {
    text_label: Option<String>,
    image_location: Option<String>,
    num_positive: Option<f32>,
    num_total: Option<f32>,
    positivity: Option<f32>,
}

impl RegionInfo {
    /// Make new RegionInfo with fully specified Options
    fn new() -> Self {
        Self { text_label: None, positivity: None, num_positive: None, num_total: None, image_location: None}
    }
    
    /// Get text label
    fn text_label(&self) -> Option<&String> {
        self.text_label.as_ref()
    }
    
    /// Get positivity
    fn positivity(&self) -> Option<f32> {
        self.positivity
    }
    
    /// Get number pixels positive
    fn num_positive(&self) -> Option<f32> {
        self.num_positive
    }

    /// Get total number of non-background pixels
    fn num_total(&self) -> Option<f32> {
        self.num_total
    }

    /// Set new text label
    fn set_text_label(&mut self, text_label: Option<String>) {
        self.text_label = text_label;
    }
    
    /// Set number positive
    fn set_num_positive(&mut self, num_pos: Option<f32>) {
        self.num_positive = num_pos;
    }

    /// Set number total
    fn set_num_total(&mut self, num_total: Option<f32>) {
        self.num_total = num_total;
    }
    /// Set positivity
    fn set_positivity(&mut self, positivity: Option<f32>) {
        self.positivity = positivity;
    }
    
    fn image_location(&self) -> Option<&String> {
        self.image_location.as_ref()
    }
    
    fn set_image_location(&mut self, image_location: Option<String>) {
        self.image_location = image_location;
    } 

}

/// Try to open and real a XML file using pre-defined structure
pub fn parse_xml(path: &path::Path) -> Annotations {
    // Read file into string and ignore any errors
    let xml = read_to_string(path).unwrap_or_default();
    // Now convert the XML into Rust data structure 
    let annotations: Annotations = quick_xml::de::from_str(&xml).expect("Error reading XML");
    return annotations;
}

pub fn run(search_path: &path::Path) -> Result<(), Box<dyn error::Error>> {
    // Setup header
    println!("Filename,Slide name,Region ID,text label,positivity,num positive,num total");
    // Iterate through list of files in search path looking for XML files only
    for entry in search_path.read_dir().expect("Invalid search path").filter(|dirent| {
        dirent.as_ref().is_ok_and(|d|  {
            return d.path().as_path().extension().is_some_and(|e| e.to_ascii_lowercase()==OsString::from("xml"));
        })
    }) {        
        let filepath = entry?.path(); // Since this is filtered, all values of the entry iterator should have valid path() so safe to use unwrap()
        //dbg!(&filepath);

        // Read XML file into annotations structure     
        let annotations = parse_xml(&filepath);
        //dbg!(&annotations);

        // Collect information about each region
        let mut regions_info: HashMap<String, RegionInfo> = HashMap::new();
        
        // Process each annotation layer
        for layer in annotations.annotation {
            match layer.annotation_type.as_str() {                
                "4" => {
                    //dbg!(&layer);
                    // Type "4" are user-drawn regions
                    // We will extract the text label for each region identified by 'Id'
                    for r in layer.regions.region {           
                        //dbg!(&r);     
                        // Find the correct region Id to store information                   
                        regions_info.entry(r.id.clone())
                        // Or make a new region Id entry if missing
                        .or_insert(RegionInfo::new())
                        // Store the label
                        .set_text_label(Some(r.text));
                    }
                },
                "3" => {
                    // Ensure an attribute header exists
                    if let Some(attribute_header) = layer.regions.region_attribute_headers.attribute_header {
                        // Locate specific attributes of interest
                        let positivity_attrib = attribute_header.iter().find(|a| a.name.starts_with("Positivity ="));
                        let num_positive_attrib = attribute_header.iter().find(|a| a.name.starts_with("Np  ="));
                        let num_total_attrib = attribute_header.iter().find(|a| a.name.starts_with("NTotal ="));
                        // If any element is missing, we will skip the file
                        if positivity_attrib.is_none() {
                            eprintln!("Missing positivity in {}", filepath.display());
                            continue;
                        }
                        if num_positive_attrib.is_none() {
                            eprintln!("Missing number positive in {}", filepath.display());
                            continue;
                        }
                        if num_total_attrib.is_none() {
                            eprintln!("Missing number total in {}", filepath.display());
                            continue;
                        } 
                        // By now we know all selected variables are valid so unwrap them
                        let positivity_name=positivity_attrib.expect("Missing positivity attribute after is_none is false").id.clone();
                        let num_positive_name=num_positive_attrib.expect("Missing number positive attribute after is_none is false").id.clone();
                        let num_total_name=num_total_attrib.expect("Missing total number attribute after is_none is false").id.clone();
                        // Now scan through each region looking for specified attributes and store the value
                        for r in layer.regions.region {
                            //dbg!(&r);
                            // Get image location for this region (stripped down to just the filename)
                            if let Some(loc) = path::Path::new(&r.image_location.unwrap_or(String::from(""))).file_name() {
                                // Try to convert OsStr to String
                                if let Some(lp) = loc.to_str() {
                                    // Start by locating a region info for this region
                                    regions_info.entry(r.id.clone())
                                    // or alternatively make a new entry
                                    .or_insert(RegionInfo::new())
                                    // Convert result into String and return "" if unable
                                    .set_image_location(Some(lp.to_string()));
                                }                                
                            }
                            // Check first if there exists a Region Attributes section for this region
                            if let Some(region_attrib) = r.attributes.attribute {
                                // Now search through each atttribute to find the positivity attribute
                                for attrib in region_attrib {
                                    if attrib.name==positivity_name {
                                        // Find the correct region Id to store information
                                        regions_info.entry(r.id.clone())
                                        // Or make a new entry if missing
                                        .or_insert(RegionInfo::new())
                                        // Convert result into f32 and return NAN if unable
                                        .set_positivity(Some(attrib.value.trim().parse::<f32>().unwrap_or(std::f32::NAN)));
                                    }
                                    if attrib.name==num_positive_name {
                                        // Find the correct region Id to store information
                                        regions_info.entry(r.id.clone())
                                        // Or make a new entry if missing
                                        .or_insert(RegionInfo::new())
                                        // Convert result into f32 and return 0 if unable
                                        .set_num_positive(Some(attrib.value.trim().parse::<f32>().unwrap_or(std::f32::NAN)));
                                    }
                                    if attrib.name==num_total_name {
                                        // Find the correct region Id to store information
                                        regions_info.entry(r.id.clone())
                                        // Or make a new entry if missing
                                        .or_insert(RegionInfo::new())
                                        // Convert result into f32 and return 0 if unable
                                        .set_num_total(Some(attrib.value.trim().parse::<f32>().unwrap_or(std::f32::NAN)));
                                    }
                                }                                
                            }                                
                        }
                    } else {
                        eprintln!("In {}: Type 3 annotation layer is missing Region Attribute header", filepath.display());
                        continue;
                    }
                },
                // Ignore other annotation types
                &_ => {},
            }            
        }

        // Report filename, region, and information about each region
        for r in &regions_info {
            println!("{}, {}, {}, {}, {}, {}, {}", &filepath.file_name().expect("Error parsing filename from full path").to_str().expect("Unable to convert filename to string"), r.1.image_location().unwrap_or(&String::from("")), r.0, r.1.text_label().unwrap_or(&String::from("")), r.1.positivity().unwrap_or(std::f32::NAN), r.1.num_positive().unwrap_or(std::f32::NAN), r.1.num_total().unwrap_or(std::f32::NAN));
        }
        //dbg!(&regions_info);

        
        

        /*
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
        } */ 
    } 

    return Ok(());
}

/// List of annotations
#[derive(Serialize, Deserialize, Debug)]
pub struct Annotations {
    /// Scale in microns per pixel (assuming square aspect ratio)
    #[serde(rename = "@MicronsPerPixel")]
    pub microns_per_pixel: String,
    /// List of annotations
    #[serde(rename = "Annotation")]
    pub annotation: Vec<Annotation>,
}

/// An annotation layer
#[derive(Serialize, Deserialize, Debug)]
pub struct Annotation {
    /// Annotation ID
    #[serde(rename = "@Id")]
    pub id: String,
    /// Name of annotation layer
    #[serde(rename = "@Name")]
    pub name: String,
    /// Annotation type
    /// 4 = user-defined drawn regions
    /// 3 = calculated data from analysis
    #[serde(rename = "@Type")]
    pub annotation_type: String,
    /// List of annotation attributes
    #[serde(rename = "Attributes")]
    pub attributes: AnnotationAttributes,
    /// List of regions
    #[serde(rename = "Regions")]
    pub regions: Regions
}

/// A specific attribute for an annotation
#[derive(Serialize, Deserialize, Debug)]
pub struct AnnotationAttributes {
    #[serde(rename = "Attribute")]
    pub attribute: Option<Vec<AnnotationAttributesAttribute>>,
}

/// Annotation attribute details
#[derive(Serialize, Deserialize, Debug)]
pub struct AnnotationAttributesAttribute {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@Value")]
    pub value: String,
}

/// List of regions in an annotation layer
#[derive(Serialize, Deserialize, Debug)]
pub struct Regions {
    #[serde(rename = "RegionAttributeHeaders")]
    pub region_attribute_headers: RegionAttributeHeaders,
    #[serde(rename = "Region")]
    pub region: Vec<Region>,
}

/// Meta-information about region attributes common across regions (header)
#[derive(Serialize, Deserialize, Debug)]
pub struct RegionAttributeHeaders {
    #[serde(rename = "AttributeHeader")]
    pub attribute_header: Option<Vec<AttributeHeader>>,
}

/// Region attribute header details
#[derive(Serialize, Deserialize, Debug)]
pub struct AttributeHeader {
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@Name")]
    pub name: String,
}

/// Details about each region
#[derive(Serialize, Deserialize, Debug)]
pub struct Region {
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@Type")]
    pub region_type: String,
    #[serde(rename = "@Length")]
    pub length: String,
    #[serde(rename = "@Area")]
    pub area: String,
    #[serde(rename = "@LengthMicrons")]
    pub length_microns: String,
    #[serde(rename = "@AreaMicrons")]
    pub area_microns: String,
    #[serde(rename = "@Text")]
    pub text: String,
    #[serde(rename = "@NegativeROA")]
    pub negative_roa: String,
    #[serde(rename = "@Analyze")]
    pub analyze: String,
    #[serde(rename = "Attributes")]
    pub attributes: RegionAttributes,
    #[serde(rename="@ImageLocation")]
    pub image_location: Option<String>,
}

/// Region attribute
#[derive(Serialize, Deserialize, Debug)]
pub struct RegionAttributes {
    #[serde(rename = "Attribute")]
    pub attribute: Option<Vec<RegionAttributesAttribute>>,
}

/// Region attribute detail
#[derive(Serialize, Deserialize, Debug)]
pub struct RegionAttributesAttribute {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@Value")]
    pub value: String,
    #[serde(rename = "@DisplayColor")]
    pub display_color: String,
}